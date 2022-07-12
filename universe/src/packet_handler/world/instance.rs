use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::{Client, ClientManager, Entity},
    database::{attrib::Attribute, license::LicenseQuery, AttribDB, Database, LicenseDB},
    world::{World, WorldRating, WorldStatus},
};
use aw_core::{AWPacket, AWPacketVar, PacketType, ReasonCode, VarID};
use num_traits::FromPrimitive;

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
    World::send_update_to_all(&new_world, client_manager);
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
        World::send_update_to_all(&removed_world, client_manager);
    }

    let mut p = AWPacket::new(PacketType::WorldStop);

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

    World::send_update_to_all(&world, client_manager);
}
