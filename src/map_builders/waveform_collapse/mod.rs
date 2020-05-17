use super::{
    generate_voronoi_spawn_regions, get_central_starting_position,
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

mod common;
use common::*;
mod constraints;
use constraints::*;
mod solver;
use solver::*;

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    derive_from: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for WaveformCollapseBuilder {
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

impl WaveformCollapseBuilder {
    /// Generic constructor for waveform collapse.
    /// # Arguments
    /// * depth - the new map depth
    /// * derive_from - either None, or a boxed MapBuilder, as output by `random_builder`
    pub fn new(depth: i32, derive_from: Option<Box<dyn MapBuilder>>) -> Self {
        WaveformCollapseBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            derive_from,
            spawn_list: Vec::new(),
        }
    }

    /// Derives a map from a pre-existing map builder.
    /// # Arguments
    /// * new_depth - the new map depth
    /// * derive_from - either None, or a boxed MapBuilder, as output by `random_builder`
    pub fn derived_map(depth: i32, builder: Box<dyn MapBuilder>) -> WaveformCollapseBuilder {
        WaveformCollapseBuilder::new(depth, Some(builder))
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        const CHUNK_SIZE: i32 = 7;

        let pre_builder = &mut self.derive_from.as_mut().unwrap();
        pre_builder.build_map();
        self.map = pre_builder.get_map();
        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStairs {
                *t = TileType::Floor;
            }
        }
        self.take_snapshot();

        let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);
        self.take_snapshot();

        let mut tries = 1;
        loop {
            self.map = Map::new(self.map.depth);
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.take_snapshot();
            }
            self.take_snapshot();

            // If it has hit an impossible condition, try again
            if solver.possible {
                break;
            }

            tries += 1;
        }
        rltk::console::log(format!("Took {} tries to solve", tries));

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

    fn render_tile_gallery(&mut self, constraints: &Vec<MapChunk>, chunk_size: i32) {
        self.map = Map::new(self.map.depth);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < constraints.len() {
            render_pattern_to_map(&mut self.map, &constraints[counter], chunk_size, x, y);

            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;

                if y + chunk_size > self.map.height {
                    // Move to the next page
                    self.take_snapshot();
                    self.map = Map::new(self.map.depth);
                    x = 1;
                    y = 1;
                }
            }

            counter += 1;
        }
        self.take_snapshot();
    }
}
