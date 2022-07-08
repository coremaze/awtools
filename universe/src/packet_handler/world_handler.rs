use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::{Client, ClientManager, ClientType, Entity, World, WorldServerInfo, WorldStatus},
    database::{attrib::Attribute, license::LicenseQuery, AttribDB, Database, LicenseDB},
};
use aw_core::{AWPacket, AWPacketVar, PacketType, ReasonCode, VarID};

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
    let world_build = match &client.info().entity {
        Some(Entity::WorldServer(info)) => info.build,
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
    let mut entity = client.info_mut().entity.take();
    if let Some(Entity::WorldServer(server_info)) = &mut entity {
        server_info.worlds.push(World {
            name: lic.name.clone(),
            status: if world_free_entry != 0 {
                WorldStatus::Permitted
            } else {
                WorldStatus::NotPermitted
            },
            rating: world_rating,
        });
    }
    client.info_mut().entity = entity;

    p.add_var(AWPacketVar::String(
        VarID::WorldStartWorldName,
        world_name.clone(),
    ));

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

    log::info!("Starting world {p:?}");

    client.connection.send(p);
}

pub fn world_stop(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let world_exists = client_manager.get_world_by_name(&world_name).is_some();

    // Remove the world from the client
    let mut entity = client.info_mut().entity.take();
    let mut removed_worlds = false;
    if let Some(Entity::WorldServer(server_info)) = &mut entity {
        let initial_len = server_info.worlds.len();
        server_info
            .worlds
            .retain(|x| !x.name.eq_ignore_ascii_case(&world_name));
        if server_info.worlds.len() < initial_len {
            removed_worlds = true;
        }
    }
    client.info_mut().entity = entity;

    // TOOD: Remove world from clients' world list
    let mut p = AWPacket::new(PacketType::WorldStop);

    let rc = match world_exists {
        true => match removed_worlds {
            true => ReasonCode::Success,
            false => ReasonCode::NotWorldOwner,
        },
        false => ReasonCode::NoSuchWorld,
    };

    p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(p);
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
