use super::{components::*, gamelog::GameLog, Map, RunState, State, TileType, Viewshed};
use legion::prelude::*;
use rltk::{Point, Rltk, VirtualKeyCode};
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let map = gs.resources.get::<Map>().unwrap();

    let mut wants_to_melee = Vec::new();
    let mut open_doors = Vec::new();
    unsafe {
        let players_query = <(Write<Position>, Write<Viewshed>)>::query().filter(tag::<Player>());
        for (player_entity, (mut pos, mut viewshed)) in
            players_query.iter_entities_unchecked(&gs.world)
        {
            let dest_x = pos.x + delta_x;
            let dest_y = pos.y + delta_y;
            if dest_x < 0 || dest_x > map.width - 1 || dest_y < 0 || dest_y > map.height - 1 {
                return;
            }
            let dest_idx = map.xy_idx(dest_x, dest_y);

            let mut recompute_blocked = false;
            for potential_target in map.tile_content[dest_idx].iter() {
                let bystander = gs.world.get_tag::<Bystander>(*potential_target);
                let vendor = gs.world.get_tag::<Vendor>(*potential_target);
                if bystander.is_some() || vendor.is_some() {
                    if let Some(mut target_position) = gs
                        .world
                        .get_component_mut_unchecked::<Position>(*potential_target)
                    {
                        target_position.x = pos.x;
                        target_position.y = pos.y;
                        recompute_blocked = true;
                    }
                } else if let Some(_target) = gs.world.get_component::<Pools>(*potential_target) {
                    // Store attack target
                    wants_to_melee.push((player_entity, *potential_target));
                }

                if let Some(_door) = gs.world.get_component::<Door>(*potential_target) {
                    open_doors.push(*potential_target);
                    viewshed.dirty = true;
                }
            }

            // FIXME: recompute blocked tiles, but this needs to wait until spatial indexing service
            if !map.blocked[dest_idx] || recompute_blocked {
                pos.x = min(map.width - 1, max(0, dest_x));
                pos.y = min(map.height - 1, max(0, dest_y));

                viewshed.dirty = true;

                // Update Player position resource
                let mut p_pos = gs.resources.get_mut::<Point>().unwrap();
                p_pos.x = pos.x;
                p_pos.y = pos.y;
            }
        }
    }

    // Add WantsToMelee component to all stored entities
    for (entity, target) in wants_to_melee.iter() {
        gs.world
            .add_component(*entity, WantsToMelee { target: *target })
            .expect("Add target failed");
    }

    // open doors
    for entity in open_doors.iter() {
        if let Some(mut door) = gs.world.get_component_mut::<Door>(*entity) {
            door.open = true;
        }
        gs.world
            .remove_tag::<BlocksVisibility>(*entity)
            .expect("Cannot remove BlocksVisibility tag");
        gs.world
            .remove_tag::<BlocksTile>(*entity)
            .expect("Cannot remove BlocksTile tag");
        if let Some(mut glyph) = gs.world.get_component_mut::<Renderable>(*entity) {
            glyph.glyph = rltk::to_cp437('/');
        }
    }
}

pub fn try_next_level(resources: &mut Resources) -> bool {
    let player_pos = resources.get::<Point>().unwrap();
    let map = resources.get::<Map>().unwrap();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = resources.get_mut::<GameLog>().unwrap();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
}

fn get_item(gs: &mut State) {
    let player_pos = gs.resources.get::<Point>().unwrap();
    let player_entity = gs.resources.get::<Entity>().unwrap();
    let mut gamelog = gs.resources.get_mut::<GameLog>().unwrap();

    let mut target_item = None;
    let query = Read::<Position>::query().filter(tag::<Item>());
    for (item_entity, position) in query.iter_entities(&gs.world) {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => gs
            .world
            .add_component(
                *player_entity,
                WantsToPickupItem {
                    collected_by: *player_entity,
                    item,
                },
            )
            .expect("Unable to insert want to pickup"),
    }
}

fn skip_turn(gs: &mut State) -> RunState {
    let player_entity = gs.resources.get::<Entity>().unwrap();
    let map = gs.resources.get::<Map>().unwrap();

    let mut can_heal = true;
    {
        let viewshed = gs.world.get_component::<Viewshed>(*player_entity).unwrap();
        for tile in viewshed.visible_tiles.iter() {
            let idx = map.xy_idx(tile.x, tile.y);
            for entity in map.tile_content[idx].iter() {
                if let Some(_monster) = gs.world.get_tag::<Monster>(*entity) {
                    can_heal = false;
                }
            }
        }
    }

    if let Some(hc) = gs.world.get_component::<HungerClock>(*player_entity) {
        match hc.state {
            HungerState::Hungry | HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    if can_heal {
        if let Some(mut player_stats) = gs.world.get_component_mut::<Pools>(*player_entity) {
            player_stats.hit_points.current = i32::min(
                player_stats.hit_points.current + 1,
                player_stats.hit_points.max,
            );
        }
    }

    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, gs)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, gs)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, gs)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, gs)
            }
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(1, -1, gs),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, gs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, gs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, gs),

            VirtualKeyCode::G => get_item(gs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,

            VirtualKeyCode::Escape => return RunState::SaveGame,

            // Level changes
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.resources) {
                    return RunState::NextLevel;
                }
            }

            // Skip turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(gs),

            _ => return RunState::PlayerTurn,
        },
    }
    RunState::PlayerTurn
}
