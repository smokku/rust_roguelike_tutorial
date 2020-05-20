use super::{BuilderMap, MetaMapBuilder, TileType};
use rltk::RandomNumberGenerator;

pub struct DistantExit {}

impl MetaMapBuilder for DistantExit {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DistantExit {
    #[allow(dead_code)]
    pub fn new() -> Box<DistantExit> {
        Box::new(DistantExit {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Find all the tiles we can reach from the starting point
        let starting_pos = build_data.starting_position.as_ref().unwrap();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);
        build_data.map.populate_blocked();
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            build_data.map.width,
            build_data.map.height,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        let mut farthest_tile = 0;
        let mut farthest_tile_distance = 0.0f32;
        for (i, tile) in build_data.map.tiles.iter().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start != std::f32::MAX {
                    // If it is further away, move the exit
                    if distance_to_start > farthest_tile_distance {
                        farthest_tile = i;
                        farthest_tile_distance = distance_to_start;
                    }
                }
            }
        }

        // Place the stairs
        build_data.map.tiles[farthest_tile] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
