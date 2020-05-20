use super::{BuilderMap, MetaMapBuilder, Position, TileType};
use rltk::RandomNumberGenerator;

pub enum XStart {
    LEFT,
    CENTER,
    RIGHT,
}

pub enum YStart {
    TOP,
    CENTER,
    BOTTOM,
}

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
    pub fn new(x: XStart, y: YStart) -> Box<Self> {
        Box::new(AreaStartingPosition { x, y })
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seed_x;
        let seed_y;

        match self.x {
            XStart::LEFT => seed_x = 1,
            XStart::CENTER => seed_x = build_data.map.width / 2,
            XStart::RIGHT => seed_x = build_data.map.width - 2,
        }
        match self.y {
            YStart::TOP => seed_y = 1,
            YStart::CENTER => seed_y = build_data.map.height / 2,
            YStart::BOTTOM => seed_y = build_data.map.height - 2,
        }

        let mut available_floors = Vec::new();
        for (idx, tile_type) in build_data.map.tiles.iter().enumerate() {
            if *tile_type == TileType::Floor {
                available_floors.push((
                    idx,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        rltk::Point::new(
                            idx as i32 % build_data.map.width,
                            idx as i32 / build_data.map.width,
                        ),
                        rltk::Point::new(seed_x, seed_y),
                    ),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let starting_idx = available_floors[0].0;
        let start_x = starting_idx as i32 % build_data.map.width;
        let start_y = starting_idx as i32 / build_data.map.width;

        build_data.starting_position = Some(Position {
            x: start_x,
            y: start_y,
        });
    }
}
