use super::{BuilderMap, MetaMapBuilder, TileType};
use rltk::RandomNumberGenerator;

pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
    #[allow(dead_code)]
    pub fn new() -> Box<Self> {
        Box::new(CullUnreachable {})
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
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                if distance_to_start == std::f32::MAX {
                    // We can't get to this tile, so we'll make it a wall
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
