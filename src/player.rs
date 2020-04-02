use super::{Map, Player, Position, TileType, Viewshed};
use legion::prelude::*;
use rltk::{Rltk, VirtualKeyCode};
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World, resources: &Resources) {
    let map = resources.get::<Map>().unwrap();

    let query = <(Write<Position>, Write<Viewshed>)>::query().filter(tag::<Player>());
    for (mut pos, mut viewshed) in query.iter_mut(world) {
        let destination_x = pos.x + delta_x;
        let destination_y = pos.y + delta_y;
        let destination_idx = map.xy_idx(destination_x, destination_y);

        if map.tiles[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, destination_x));
            pos.y = min(49, max(0, destination_y));

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(world: &mut World, resources: &Resources, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, world, resources)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, world, resources)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, world, resources)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, world, resources)
            }
            _ => {}
        },
    }
}
