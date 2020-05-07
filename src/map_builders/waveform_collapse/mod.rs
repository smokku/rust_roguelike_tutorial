use super::{
    generate_voronoi_spawn_regions, get_central_starting_position,
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct WaveformCollapseBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
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

    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources) {
        for (_id, area) in self.noise_areas.iter() {
            spawner::spawn_region(world, resources, area, self.map.depth);
        }
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
    pub fn new(depth: i32) -> Self {
        WaveformCollapseBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

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
    }
}
