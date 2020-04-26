use super::{
    gamelog::GameLog, CombatStats, HungerClock, HungerState, Item, Map, Monster, Player, Position,
    RunState, State, TileType, Viewshed, WantsToMelee, WantsToPickupItem,
};
use legion::prelude::*;
use rltk::{Point, Rltk, VirtualKeyCode};
use std::cmp::{max, min};
use std::collections::HashMap;

pub fn try_move_player(delta_x: i32, delta_y: i32, gs: &mut State) {
    let map = gs.resources.get::<Map>().unwrap();

    // Pull all CombatStats for reference later
    // FIXME: This is required as you cannot access random parts
    // of ECS during query iteration - world is already borrowed.
    // This is a sub-optimal pattern and should be implemented some other way.
    // ECS is designed to work in carefully crafted chunks of data, withouts
    // accessing data all over the place.
    let mut combat_stats = HashMap::new();
    let query = Read::<CombatStats>::query();
    for (entity, cs) in query.iter_entities(&gs.world) {
        combat_stats.insert(entity, *cs);
    }

    let mut wants_to_melee = Vec::new();
    let query = <(Write<Position>, Write<Viewshed>)>::query().filter(tag::<Player>());
    for (entity, (mut pos, mut viewshed)) in query.iter_entities_mut(&mut gs.world) {
        let destination_x = pos.x + delta_x;
        let destination_y = pos.y + delta_y;
        let destination_idx = map.xy_idx(destination_x, destination_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let cs = combat_stats.get(&*potential_target);
            if let Some(_cs) = cs {
                // Store attack target
                wants_to_melee.push((entity, *potential_target));
                continue; // So we don't move after attacking
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, destination_x));
            pos.y = min(49, max(0, destination_y));

            viewshed.dirty = true;

            // Update Player position resource
            let mut p_pos = gs.resources.get_mut::<Point>().unwrap();
            p_pos.x = pos.x;
            p_pos.y = pos.y;
        }
    }

    // Add WantsToMelee component to all stored entities
    for (entity, target) in wants_to_melee.iter() {
        gs.world
            .add_component(*entity, WantsToMelee { target: *target })
            .expect("Add target failed");
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
        if let Some(mut player_hp) = gs.world.get_component_mut::<CombatStats>(*player_entity) {
            player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
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
