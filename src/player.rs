use super::{CombatStats, Map, Player, Position, RunState, State, Viewshed, WantsToMelee};
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

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::Paused, // Nothing happened
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
            _ => return RunState::Paused,
        },
    }
    RunState::Running
}
