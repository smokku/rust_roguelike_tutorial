use super::{
    tile_walkable, AreaStartingPosition, BuilderChain, BuilderMap, CellularAutomataBuilder,
    CullUnreachable, MetaMapBuilder, TileType, VoronoiSpawning, XStart, YStart,
};
use crate::a_star_search;
use rltk::{Point, RandomNumberGenerator};

pub fn forest_builder(
    depth: i32,
    width: i32,
    height: i32,
    _rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));
    chain.with(VoronoiSpawning::new());
    chain.with(YellowBrickRoad::new());
    chain
}

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad {})
    }

    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors = Vec::new();
        for (idx, tile_type) in build_data.map.tiles.iter().enumerate() {
            if tile_walkable(*tile_type) {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                available_floors.push((
                    idx,
                    rltk::DistanceAlg::PythagorasSquared
                        .distance2d(Point::new(x, y), Point::new(seed_x, seed_y)),
                ))
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let end_idx = available_floors[0].0;
        let end_x = end_idx as i32 % build_data.map.width;
        let end_y = end_idx as i32 / build_data.map.width;
        (end_x, end_y)
    }

    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 0 || x >= build_data.map.width || y < 0 || y >= build_data.map.height {
            return;
        }

        let idx = build_data.map.xy_idx(x, y);
        if build_data.map.tiles[idx] != TileType::DownStairs {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap().clone();

        let (end_x, end_y) = self.find_exit(
            build_data,
            build_data.map.width - 1,
            build_data.map.height / 2,
        );
        let end_idx = build_data.map.xy_idx(end_x, end_y);
        build_data.map.tiles[end_idx] = TileType::DownStairs;

        build_data.map.populate_blocked();
        if let Some(result) = a_star_search(
            Point::new(starting_pos.x, starting_pos.y),
            Point::new(end_x, end_y),
            0.0,
            &build_data.map,
        ) {
            for p in result.0.iter() {
                let x = p.x;
                let y = p.y;
                self.paint_road(build_data, x, y);
                self.paint_road(build_data, x - 1, y);
                self.paint_road(build_data, x + 1, y);
                self.paint_road(build_data, x, y - 1);
                self.paint_road(build_data, x, y + 1);
            }
        }
        build_data.take_snapshot();

        // Place exit
        let exit_dir = rng.roll_dice(1, 2);
        let (seed_x, seed_y, stream_startx, stream_starty) = if exit_dir == 1 {
            (build_data.map.width - 1, 0, 0, build_data.map.height - 1)
        } else {
            (build_data.map.width - 1, build_data.map.height - 1, 0, 0)
        };

        let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
        let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;

        let (stream_x, stream_y) = self.find_exit(build_data, stream_startx, stream_starty);
        let stream = a_star_search(
            Point::new(stairs_x, stairs_y),
            Point::new(stream_x, stream_y),
            0.0,
            &build_data.map,
        );
        // FIXME: First trace the stream, then draw the road.
        // Now the stream flows using road tiles.
        for tile in stream.unwrap().0.iter() {
            let idx = build_data.map.xy_idx(tile.x, tile.y);
            if build_data.map.tiles[idx] == TileType::Floor {
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();
    }
}
