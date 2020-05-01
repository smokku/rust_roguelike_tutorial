use super::{
    apply_horizontal_tunnel, apply_room_to_map, apply_vertical_tunnel, spawner, Map, MapBuilder,
    Position, Rect, TileType,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder;

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, depth: i32) -> (Map, Position) {
        let mut map = Map::new(depth);
        let player_pos = SimpleMapBuilder::rooms_and_corridors(&mut map);
        (map, player_pos)
    }

    fn spawn_entities(&mut self, map: &mut Map, world: &mut World, resources: &mut Resources) {
        // Spawn bad guys and items
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(world, resources, room, map.depth);
        }
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Self {
        SimpleMapBuilder {}
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    fn rooms_and_corridors(map: &mut Map) -> Position {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersects(other_room) {
                    ok = false;
                    break;
                }
            }
            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                        apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    } else {
                        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            };
        }

        let stairs_position = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
        map.tiles[stairs_idx] = TileType::DownStairs;

        let start_pos = map.rooms[0].center();
        Position {
            x: start_pos.0,
            y: start_pos.1,
        }
    }
}
