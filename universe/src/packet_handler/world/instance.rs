use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::ClientInfo,
    database::{attrib::Attribute, license::LicenseQuery, AttribDB, DatabaseResult, LicenseDB},
    get_conn, get_conn_mut,
    tabs::regenerate_world_list,
    universe_connection::UniverseConnectionID,
    world::{World, WorldRating},
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use num_traits::FromPrimitive;

pub fn world_start(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "world_start");

    let Some(client) = &conn.client else {
        return;
    };

    let ClientInfo::WorldServer(world_server) = client else {
        return;
    };

    let world_build = world_server.build;

    let Some(world_name) = packet.get_string(VarID::WorldName) else {
        return;
    };

    let Some(world_password) = packet.get_string(VarID::WorldLicensePassword) else {
        return;
    };

    let Some(world_rating) = packet.get_byte(VarID::WorldRating) else {
        return;
    };

    let Some(world_free_entry) = packet.get_byte(VarID::WorldFreeEntry) else {
        return;
    };

    let mut p = AWPacket::new(PacketType::WorldStart);

    p.add_string(VarID::WorldName, world_name.clone());

    let lic = match validate_world(server, world_build, &world_name, &world_password) {
        Ok(x) => x,
        Err(rc) => {
            log::info!("Unable to start world: {rc:?}");
            p.add_int(VarID::ReasonCode, rc as i32);
            conn.send(p);
            return;
        }
    };

    // Don't let clients start a world twice
    if server.connections.get_world_by_name(&lic.name).is_some() {
        log::info!(
            "{:?} attempted to start a world {:?} twice.",
            &conn,
            &world_name
        );
        p.add_int(VarID::ReasonCode, ReasonCode::WorldAlreadyStarted as i32);
        conn.send(p);
        return;
    }

    let new_world = World {
        name: lic.name.clone(),
        free_entry: world_free_entry != 0,
        world_size: lic.world_size,
        max_users: lic.users,
        rating: WorldRating::from_u8(world_rating).unwrap_or_default(),
        user_count: 0,
    };

    p.add_uint(VarID::WorldLicenseExpiration, lic.expiration);
    p.add_uint(VarID::WorldLicenseUsers, lic.users);
    p.add_uint(VarID::WorldLicenseRange, lic.world_size);
    p.add_uint(VarID::WorldLicenseVoip, lic.voip);
    p.add_uint(VarID::WorldLicensePlugins, lic.plugins);

    p.add_int(VarID::ReasonCode, ReasonCode::Success as i32);

    conn.send(p);

    add_world_to_world_server(server, cid, new_world);

    // Add information about the new world to everyone's world list
    for cid in server.connections.cids() {
        regenerate_world_list(server, cid)
    }
}

fn add_world_to_world_server(server: &mut UniverseServer, cid: UniverseConnectionID, world: World) {
    let conn = get_conn_mut!(server, cid, "add_world_to_world_server");

    let Some(client) = &mut conn.client else {
        return;
    };

    let ClientInfo::WorldServer(world_server) = client else {
        return;
    };

    world_server.worlds.push(world);
}

pub fn world_stop(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let world_name = match packet.get_string(VarID::WorldName) {
        Some(x) => x,
        None => return,
    };

    let world_exists = server.connections.get_world_by_name(&world_name).is_some();

    let conn = get_conn_mut!(server, cid, "world_stop");

    let Some(ClientInfo::WorldServer(world_server)) = &mut conn.client else {
        return;
    };

    // Remove the world from the client
    log::trace!("Before remove: {world_server:?}");
    let removed_world = world_server.remove_world(&world_name);
    log::trace!("After remove: {world_server:?}");
    log::trace!("{removed_world:?}");

    let rc = match world_exists {
        true => match removed_world {
            Some(_) => ReasonCode::Success,
            None => ReasonCode::NotWorldOwner,
        },
        false => ReasonCode::NoSuchWorld,
    };

    let mut p = AWPacket::new(PacketType::WorldStop);

    p.add_int(VarID::ReasonCode, rc as i32);

    conn.send(p);

    // Remove world from clients' world list
    for cid in server.connections.cids() {
        regenerate_world_list(server, cid)
    }
}

fn validate_world(
    server: &UniverseServer,
    world_build: i32,
    name: &str,
    pass: &str,
) -> Result<LicenseQuery, ReasonCode> {
    let attribs = match server.database.attrib_get() {
        DatabaseResult::Ok(attribs) => attribs,
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

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

    let world_lic = match server.database.license_by_name(name) {
        DatabaseResult::Ok(Some(lic)) => lic,
        DatabaseResult::Ok(None) => return Err(ReasonCode::InvalidWorld),
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

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

pub fn world_stats_update(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let Some(world_rating) = packet.get_byte(VarID::WorldRating) else {
        return;
    };

    let Some(world_free_entry) = packet.get_byte(VarID::WorldFreeEntry) else {
        return;
    };

    let Some(user_count) = packet.get_uint(VarID::WorldUsers) else {
        return;
    };

    let Some(world_name) = packet.get_string(VarID::WorldName) else {
        return;
    };

    let conn = get_conn_mut!(server, cid, "world_stats_update");

    let Some(ClientInfo::WorldServer(world_server)) = &mut conn.client else {
        return;
    };

    let Some(world) = world_server.get_world_mut(&world_name) else {
        return;
    };

    world.rating = WorldRating::from_u8(world_rating).unwrap_or_default();
    world.free_entry = world_free_entry != 0;
    world.user_count = user_count;

    // Change world information in everyone's world list
    for cid in server.connections.cids() {
        regenerate_world_list(server, cid)
    }
}
