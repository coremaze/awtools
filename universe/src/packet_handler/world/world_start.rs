use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;

use crate::{
    database::{attrib::Attribute, license::LicenseQuery, AttribDB, LicenseDB},
    get_conn, get_conn_mut,
    tabs::regenerate_world_list,
    timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID,
    world::{World, WorldRating},
    UniverseServer,
};

#[derive(Debug)]
enum WorldStartParamsError {
    Name,
    Password,
    Rating,
    FreeEntry,
}

struct WorldStartParams {
    world_name: String,
    world_password: String,
    world_rating: WorldRating,
    world_free_entry: bool,
}

impl TryFrom<&AWPacket> for WorldStartParams {
    type Error = WorldStartParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let world_name = value
            .get_string(VarID::WorldName)
            .ok_or(WorldStartParamsError::Name)?;
        let world_password = value
            .get_string(VarID::WorldLicensePassword)
            .ok_or(WorldStartParamsError::Password)?;
        let world_rating = value
            .get_byte(VarID::WorldRating)
            .and_then(WorldRating::from_u8)
            .ok_or(WorldStartParamsError::Rating)?;
        let world_free_entry = value
            .get_byte(VarID::WorldFreeEntry)
            .map(|free_entry| free_entry != 0)
            .ok_or(WorldStartParamsError::FreeEntry)?;

        Ok(WorldStartParams {
            world_name,
            world_password,
            world_rating,
            world_free_entry,
        })
    }
}

pub fn world_start(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "world_start");

    let Some(world_server) = conn.world_server() else {
        return;
    };

    let world_build = world_server.build;

    let params = match WorldStartParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete world start: {why:?}");
            return;
        }
    };

    let mut p = AWPacket::new(PacketType::WorldStart);

    p.add_string(VarID::WorldName, params.world_name.clone());

    let lic = match validate_world(
        server,
        world_build,
        &params.world_name,
        &params.world_password,
    ) {
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
            &params.world_name
        );
        p.add_int(VarID::ReasonCode, ReasonCode::WorldAlreadyStarted.into());
        conn.send(p);
        return;
    }

    let new_world = World {
        name: lic.name.clone(),
        free_entry: params.world_free_entry,
        world_size: lic.world_size,
        max_users: lic.users,
        rating: params.world_rating,
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

    let Some(world_server) = conn.world_server_mut() else {
        return;
    };

    world_server.worlds.push(world);
}

fn validate_world(
    server: &UniverseServer,
    world_build: u32,
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
        .parse::<u32>()
        .unwrap_or(0);

    let latest_world_build = attribs
        .get(&Attribute::MinimumWorld)
        .unwrap_or(&String::new())
        .parse::<u32>()
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

    let now = unix_epoch_timestamp_u32();

    // Check if world is expired
    if world_lic.expiration != 0 && world_lic.expiration < now {
        return Err(ReasonCode::WorldExpired);
    }

    Ok(world_lic)
}
