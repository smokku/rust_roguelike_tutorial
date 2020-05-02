use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE: i32 = 8;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources) {
        // We need to rewrite this, too.
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // First we completely randomize the map, setting 55% of it tto be floor.
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                self.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                };
            }
        }
        self.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _i in 0..15 {
            let mut new_tiles = vec![TileType::Wall; self.map.tiles.len()];

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - self.map.width as usize + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + self.map.width as usize + 1] == TileType::Wall {
                        neighbors += 1;
                    }

                    new_tiles[idx] = if neighbors > 4 || neighbors == 0 {
                        TileType::Wall
                    } else {
                        TileType::Floor
                    }
                }
            }

            self.map.tiles = new_tiles;
            self.take_snapshot();
        }

        // Find a starting point;
        // start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx;
        while {
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
            self.map.tiles[start_idx] != TileType::Floor
        } {
            self.starting_position.x -= 1;
        }

        // Find all the tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let mut exit_tile = 0;
        let mut exit_tile_distance = 0.0f32;
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start == std::f32::MAX {
                    // We can't get to this tile, so we'll make it a wall
                    *tile = TileType::Wall;
                } else {
                    // if it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile_distance {
                        exit_tile = i;
                        exit_tile_distance = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();
    }
}
