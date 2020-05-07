use super::{
    generate_voronoi_spawn_regions, get_central_starting_position,
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone)]
pub enum DistanceAlgorithm {
    Pythagoras,
    Manhattan,
    Chebyshev,
}

pub struct VoronoiCellBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl MapBuilder for VoronoiCellBuilder {
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

impl VoronoiCellBuilder {
    pub fn new(depth: i32) -> Self {
        VoronoiCellBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        }
    }

    pub fn manhattan(new_depth: i32) -> Self {
        VoronoiCellBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        }
    }

    pub fn chebyshev(new_depth: i32) -> Self {
        VoronoiCellBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Make a Voronoi diagram. We'll do this the hard way to learn about the technique!
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width - 1);
            let vy = rng.roll_dice(1, self.map.height - 1);
            let vidx = self.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0); voronoi_seeds.len()];
        let mut voronoi_membership = vec![0; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % self.map.width;
            let y = i as i32 / self.map.width;

            for (seed, (_, pos)) in voronoi_seeds.iter().enumerate() {
                let distance;
                match self.distance_algorithm {
                    DistanceAlgorithm::Pythagoras => {
                        distance = rltk::DistanceAlg::PythagorasSquared
                            .distance2d(rltk::Point::new(x, y), *pos);
                    }
                    DistanceAlgorithm::Manhattan => {
                        distance =
                            rltk::DistanceAlg::Manhattan.distance2d(rltk::Point::new(x, y), *pos);
                    }
                    DistanceAlgorithm::Chebyshev => {
                        distance =
                            rltk::DistanceAlg::Chebyshev.distance2d(rltk::Point::new(x, y), *pos);
                    }
                }
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = self.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[self.map.xy_idx(x - 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x + 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y - 1)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y + 1)] != my_seed {
                    neighbors += 1;
                }

                if neighbors < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }
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
    }
}
