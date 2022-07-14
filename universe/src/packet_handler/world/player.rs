use crate::{
    client::{Client, ClientManager, Entity},
    database::Database,
    packet_handler::update_contacts_of_user,
};
use aw_core::{AWPacket, AWPacketVar, PacketType, ReasonCode, VarID};

pub fn identify(
    client: &Client,
    packet: &AWPacket,
    client_manager: &ClientManager,
    database: &Database,
) {
    let mut p = AWPacket::new(PacketType::Identify);

    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => {
            log::info!("Failed to identify player because no world name was provided");
            return;
        }
    };

    let nonce = match packet.get_data(VarID::WorldUserNonce) {
        Some(x) => x,
        None => {
            log::info!("Failed to identify player because no user nonce was provided");
            return;
        }
    };

    let session_id = match packet.get_int(VarID::SessionID) {
        Some(x) => x,
        None => {
            log::info!("Failed to identify player because no session id was provided");
            return;
        }
    };

    let player_ip = match packet.get_uint(VarID::IdentifyUserIP) {
        Some(x) => x,
        None => {
            log::info!("Failed to identify player because no user ip was provided");
            return;
        }
    };

    let player_port = match packet.get_int(VarID::PlayerPort) {
        Some(x) => x,
        None => {
            log::info!("Failed to identify player because no port was provided");
            return;
        }
    };

    let world = match &client.info().entity {
        Some(Entity::WorldServer(w)) => match w.get_world(&world_name) {
            Some(w) => w.clone(),
            None => {
                log::info!(
                    "Failed to identify player because the world server does not own the world"
                );
                return;
            }
        },
        _ => return,
    };

    let mut rc = ReasonCode::NoSuchSession;
    let mut changed_cit_id: Option<u32> = None;

    if let Some(user_client) = client_manager.get_client_by_session_id(session_id as u16) {
        if let Some(Entity::Player(user_ent)) = &mut user_client.info_mut().entity {
            if let Some(user_nonce) = user_ent.nonce {
                if user_nonce.to_vec() == nonce {
                    // Not currently checking IP address or port
                    p.add_var(AWPacketVar::String(VarID::WorldStartWorldName, world_name));
                    p.add_var(AWPacketVar::Int(VarID::SessionID, session_id));
                    p.add_var(AWPacketVar::Uint(VarID::IdentifyUserIP, player_ip));
                    p.add_var(AWPacketVar::Int(VarID::PlayerPort, player_port));
                    p.add_var(AWPacketVar::Uint(
                        VarID::LoginID,
                        user_ent.citizen_id.unwrap_or(0),
                    ));
                    p.add_var(AWPacketVar::Int(VarID::BrowserBuild, user_ent.build));
                    p.add_var(AWPacketVar::String(
                        VarID::LoginUsername,
                        user_ent.username.clone(),
                    ));
                    p.add_var(AWPacketVar::Uint(
                        VarID::PrivilegeUserID,
                        user_ent.effective_privilege(),
                    ));

                    user_ent.world = Some(world.name);

                    changed_cit_id = user_ent.citizen_id;

                    rc = ReasonCode::Success;
                }
            }
        }
    }

    p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(p);

    if let Some(citizen_id) = changed_cit_id {
        // Update the user's friends to tell them this user is in a new world
        update_contacts_of_user(citizen_id, database, client_manager);
    }
}
