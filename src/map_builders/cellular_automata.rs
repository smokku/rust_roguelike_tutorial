use super::{
    generate_voronoi_spawn_regions, get_central_starting_position,
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    spawn_list: Vec<(usize, String)>,
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

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
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
    pub fn new(depth: i32) -> Self {
        CellularAutomataBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            spawn_list: Vec::new(),
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

        // Set a central starting point
        self.starting_position = get_central_starting_position(&self.map);

        // Find all tiles we can reach from the starting point
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // Spawn the entities
        for (_id, area) in self.noise_areas.iter() {
            spawner::spawn_region(
                &self.map,
                &mut rng,
                area,
                self.map.depth,
                &mut self.spawn_list,
            );
        }
    }
}
