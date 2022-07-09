use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::{
        Client, ClientManager, ClientType, Entity, World, WorldRating, WorldServerInfo,
        WorldStatus,
    },
    database::{attrib::Attribute, license::LicenseQuery, AttribDB, Database, LicenseDB},
};
use aw_core::{AWPacket, AWPacketGroup, AWPacketVar, PacketType, ReasonCode, VarID};
use num_traits::FromPrimitive;

pub fn world_server_start(client: &Client, packet: &AWPacket) {
    if let Some(client_type) = client.info().client_type {
        log::warn!(
            "A client who already has type {:?} tried to start a world server.",
            client_type
        );
        return;
    }

    let _browser_version = packet.get_int(VarID::BrowserVersion);
    let world_build = packet.get_int(VarID::WorldBuild);
    let world_port = packet.get_int(VarID::WorldPort);

    if let (Some(world_build), Some(world_port)) = (world_build, world_port) {
        let client_entity = Entity::WorldServer(WorldServerInfo {
            build: world_build,
            server_port: world_port as u16,
            worlds: Vec::new(),
        });

        client.info_mut().client_type = Some(ClientType::World);
        client.info_mut().entity = Some(client_entity);

        log::info!("World server {} connected.", client.addr.ip());
    }
}

pub fn world_start(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    let (world_build, world_port) = match &client.info().entity {
        Some(Entity::WorldServer(info)) => (info.build, info.server_port),
        _ => {
            return;
        }
    };

    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let world_password = match packet.get_string(VarID::WorldLicensePassword) {
        Some(x) => x,
        None => return,
    };

    let world_rating = match packet.get_byte(VarID::WorldRating) {
        Some(x) => x,
        None => return,
    };

    let world_free_entry = match packet.get_byte(VarID::WorldFreeEntry) {
        Some(x) => x,
        None => return,
    };

    let mut p = AWPacket::new(PacketType::WorldStart);

    p.add_var(AWPacketVar::String(
        VarID::WorldStartWorldName,
        world_name.clone(),
    ));

    let lic = match validate_world(world_build, &world_name, &world_password, database) {
        Ok(x) => x,
        Err(rc) => {
            log::info!("Unable to start world: {rc:?}");
            p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
            client.connection.send(p);
            return;
        }
    };

    // Don't let clients start a world twice
    if client_manager.get_world_by_name(&lic.name).is_some() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::WorldAlreadyStarted as i32,
        ));
        client.connection.send(p);
        return;
    }

    // Add a new world to the client's list of worlds
    let new_world = World {
        name: lic.name.clone(),
        status: WorldStatus::from_free_entry(world_free_entry),
        rating: WorldRating::from_u8(world_rating).unwrap_or_default(),
        ip: client.addr.ip(),
        port: world_port,
        max_users: lic.users,
        world_size: lic.world_size,
        user_count: 0,
    };

    let mut entity = client.info_mut().entity.take();
    if let Some(Entity::WorldServer(server_info)) = &mut entity {
        server_info.worlds.push(new_world.clone());
    }
    client.info_mut().entity = entity;

    p.add_var(AWPacketVar::Uint(
        VarID::WorldLicenseExpiration,
        lic.expiration,
    ));
    p.add_var(AWPacketVar::Uint(VarID::WorldLicenseUsers, lic.users));
    p.add_var(AWPacketVar::Uint(VarID::WorldLicenseRange, lic.world_size));
    p.add_var(AWPacketVar::Uint(VarID::WorldLicenseVoip, lic.voip));
    p.add_var(AWPacketVar::Uint(VarID::WorldLicensePlugins, lic.plugins));

    p.add_var(AWPacketVar::Int(
        VarID::ReasonCode,
        ReasonCode::Success as i32,
    ));

    client.connection.send(p);

    // Send update about new world to all players
    send_single_world_update(&new_world, client_manager);
}

pub fn world_server_hide_all(server: &mut WorldServerInfo) {
    for world in &mut server.worlds {
        world.status = WorldStatus::Hidden;
    }
}

pub fn send_world_updates(worlds: &[World], client_manager: &ClientManager) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs();

    let world_packets = worlds
        .iter()
        .map(|x| x.make_list_packet())
        .collect::<Vec<AWPacket>>();

    // Group packets into larger transmissions for efficiency
    let mut groups: Vec<AWPacketGroup> = Vec::new();
    let mut group = AWPacketGroup::new();

    for world_packet in world_packets {
        if let Err(p) = group.push(world_packet) {
            groups.push(group);
            group = AWPacketGroup::new();

            let mut more = AWPacket::new(PacketType::WorldListResult);
            // Yes, expect another WorldList packet from the server
            more.add_var(AWPacketVar::Byte(VarID::WorldListMore, 1));
            more.add_var(AWPacketVar::Uint(VarID::WorldList3DayUnknown, now as u32));
            group.push(more).ok();
            group.push(p).ok();
        }
    }

    // Send packet indicating that the server is done
    let mut p = AWPacket::new(PacketType::WorldListResult);
    p.add_var(AWPacketVar::Byte(VarID::WorldListMore, 0));
    p.add_var(AWPacketVar::Uint(VarID::WorldList3DayUnknown, now as u32));

    if let Err(p) = group.push(p) {
        groups.push(group);
        group = AWPacketGroup::new();
        group.push(p).ok();
    }

    groups.push(group);

    // Send update to all players
    for client in client_manager.clients() {
        if let Some(Entity::Player(_)) = client.info().entity {
            for group in &groups {
                client.connection.send_group(group.clone());
            }
        }
    }
}

pub fn world_server_update_all(server: &WorldServerInfo, client_manager: &ClientManager) {
    send_world_updates(&server.worlds, client_manager);
}

pub fn world_stop(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let world_exists = client_manager.get_world_by_name(&world_name).is_some();

    // Remove the world from the client
    let mut entity = client.info_mut().entity.take();
    let mut removed_world: Option<World> = None;
    if let Some(Entity::WorldServer(server_info)) = &mut entity {
        let mut matched_index: Option<usize> = None;
        for (i, e) in server_info.worlds.iter().enumerate() {
            if e.name.eq_ignore_ascii_case(&world_name) {
                matched_index = Some(i);
                break;
            }
        }
        if let Some(i) = matched_index {
            removed_world = Some(server_info.worlds.remove(i));
        }
    }
    client.info_mut().entity = entity;

    let rc = match world_exists {
        true => match removed_world {
            Some(_) => ReasonCode::Success,
            None => ReasonCode::NotWorldOwner,
        },
        false => ReasonCode::NoSuchWorld,
    };

    // Remove world from clients' world list
    if let Some(mut removed_world) = removed_world {
        removed_world.status = WorldStatus::Hidden;
        send_single_world_update(&removed_world, client_manager);
    }

    let mut p = AWPacket::new(PacketType::WorldStop);

    p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(p);
}

fn send_single_world_update(world: &World, client_manager: &ClientManager) {
    let worlds = [world.clone()];
    send_world_updates(&worlds, client_manager);
}

fn validate_world(
    world_build: i32,
    name: &str,
    pass: &str,
    database: &Database,
) -> Result<LicenseQuery, ReasonCode> {
    let attribs = database.attrib_get()?;

    // Check to see if the world version is within universe constraints
    let minimum_world_build = attribs
        .get(&Attribute::MinimumWorld)
        .unwrap_or(&String::new())
        .parse::<i32>()
        .unwrap_or(0);

    let latest_world_build = attribs
        .get(&Attribute::MinimumWorld)
        .unwrap_or(&String::new())
        .parse::<i32>()
        .unwrap_or(0);

    if minimum_world_build != 0 && world_build < minimum_world_build {
        return Err(ReasonCode::ServerOutdated);
    }

    if latest_world_build != 0 && world_build > latest_world_build {
        return Err(ReasonCode::SdkMustUpgrade);
    }

    // TODO: Check for ejected client

    let world_lic = database
        .license_by_name(name)
        .map_err(|_| ReasonCode::InvalidWorld)?;

    // Check password
    if world_lic.password != pass {
        return Err(ReasonCode::InvalidPassword);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs();

    // Check if world is expired
    if world_lic.expiration != 0 && (world_lic.expiration as u64) < now {
        return Err(ReasonCode::WorldExpired);
    }

    Ok(world_lic)
}

pub fn identify(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
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
                log::info!("Failed to identify player because the world server does not own the world");
                return;
            },
        }
        _ => return,
    };

    let mut rc = ReasonCode::NoSuchSession;

    if let Some(user_client) = client_manager.get_client_by_session_id(session_id as u16) {
        if let Some(Entity::Player(user_ent)) = &mut user_client.info_mut().entity {
            if let Some(user_nonce) = user_ent.nonce {
                if user_nonce.to_vec() == nonce {
                    // Not currently checking IP address or port
                    p.add_var(AWPacketVar::String(
                        VarID::WorldStartWorldName,
                        world_name,
                    ));
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

                    rc = ReasonCode::Success;
                }
            }
        }
    }

    p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(p);
}

pub fn world_stats_update(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let world_rating = match packet.get_byte(VarID::WorldRating) {
        Some(x) => x,
        None => return,
    };

    let world_free_entry = match packet.get_byte(VarID::WorldFreeEntry) {
        Some(x) => x,
        None => return,
    };

    let user_count = match packet.get_uint(VarID::WorldUsers) {
        Some(x) => x,
        None => return,
    };

    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let world = if let Some(Entity::WorldServer(w)) = &mut client.info_mut().entity {
        match w.get_world_mut(&world_name) {
            Some(world) => {
                world.rating = WorldRating::from_u8(world_rating).unwrap_or_default();
                world.status = WorldStatus::from_free_entry(world_free_entry);
                world.user_count = user_count;

                world.clone()
            }
            // Return if the client doesn't own the given world
            None => return,
        }
    } else {
        return;
    };

    send_single_world_update(&world, client_manager);
}
