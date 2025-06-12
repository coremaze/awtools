use std::collections::HashMap;

use aw_core::{AWPacket, PacketType, PacketTypeResult, VarID};

use crate::{AwInstance, ObjectInfo, SdkError, SdkResult, sector_from_cell, world::World};

pub fn query(
    instance: &mut AwInstance,
    query_sector_x: i32,
    query_sector_z: i32,
) -> SdkResult<QueryResult> {
    let Some(world) = &mut instance.world else {
        return Err(SdkError::NotConnectedToWorld);
    };
    // 2d array of 3x3 sequence numbers, these indicate the progress of the query for each sector
    let mut sequence = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
    let mut current_cell: Option<(i32, i32)> = None;
    let mut objects: Vec<ObjectInfo> = Vec::new();
    gimme(world, query_sector_x, query_sector_z, &mut sequence);
    loop {
        let Some(packet) = world.connection.wait_for_packets(
            &[
                PacketType::QueryUpToDate,
                PacketType::CellBegin,
                PacketType::QueryNeedMore,
                PacketType::CellUpdate,
                PacketType::CellEnd,
            ],
            None,
        ) else {
            panic!("No packet received");
        };

        let PacketTypeResult::PacketType(packet_type) = packet.get_type() else {
            panic!("Received non-packet type: {:?}", packet);
        };

        match packet_type {
            PacketType::QueryUpToDate => {
                return Ok(QueryResult { objects });
            }
            PacketType::QueryNeedMore => {
                gimme(world, query_sector_x, query_sector_z, &mut sequence);
                continue;
            }
            PacketType::CellBegin => {
                let Some(cell_x) = packet.get_int(VarID::ObjectCellX) else {
                    return Err(SdkError::protocol("Cell x not found"));
                };
                let Some(cell_z) = packet.get_int(VarID::ObjectCellZ) else {
                    return Err(SdkError::protocol("Cell z not found"));
                };
                let Some(cell_sequence) = packet.get_int(VarID::ObjectCellSequence) else {
                    return Err(SdkError::protocol("Cell sequence not found"));
                };
                let sector_x = sector_from_cell(cell_x) - query_sector_x + 1;
                let sector_z = sector_from_cell(cell_z) - query_sector_z + 1;

                if let Some(row) = sequence.get_mut(sector_z as usize) {
                    if let Some(cell) = row.get_mut(sector_x as usize) {
                        *cell = cell_sequence;
                    } else {
                        return Err(SdkError::protocol("Invalid sector_x index"));
                    }
                } else {
                    return Err(SdkError::protocol("Invalid sector_z index"));
                }

                current_cell = Some((cell_x, cell_z));
            }
            PacketType::CellUpdate => {
                let Some((cell_x, cell_z)) = current_cell else {
                    return Err(SdkError::protocol("Cell not found"));
                };
                let mut object = ObjectInfo::try_from(&packet).unwrap();
                object.cell_x = cell_x;
                object.cell_z = cell_z;
                object.west += object.cell_x * 1000;
                object.north += object.cell_z * 1000;
                objects.push(object);
                // println!("Cell update object: {:?}", object);
            }
            PacketType::CellEnd => {
                // println!("Cell end");
                current_cell = None;
            }
            _ => {}
        }
    }
}

fn gimme(world: &mut World, query_x: i32, query_z: i32, sequence: &mut [[i32; 3]; 3]) {
    let mut packet = AWPacket::new(PacketType::ObjectQuery3x3);

    packet.add_int(VarID::ObjectQueryX, query_x);
    packet.add_int(VarID::ObjectQueryZ, query_z);

    // println!("Sequence: {:?}", sequence);
    // Var IDs collide with other things because cringe
    packet.add_int(0u16, sequence[0][0]);
    packet.add_int(1u16, sequence[0][1]);
    packet.add_int(2u16, sequence[0][2]);

    packet.add_int(3u16, sequence[1][0]);
    packet.add_int(4u16, sequence[1][1]);
    packet.add_int(5u16, sequence[1][2]);

    packet.add_int(6u16, sequence[2][0]);
    packet.add_int(7u16, sequence[2][1]);
    packet.add_int(8u16, sequence[2][2]);

    world.connection.send(packet);
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub objects: Vec<ObjectInfo>,
}
