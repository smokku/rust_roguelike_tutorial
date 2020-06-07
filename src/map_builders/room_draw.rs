use super::{BuilderMap, MetaMapBuilder, TileType};
use rltk::RandomNumberGenerator;

pub struct RoomDrawer {}

impl MetaMapBuilder for RoomDrawer {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomDrawer {
    pub fn new() -> Box<Self> {
        Box::new(RoomDrawer {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Room Drawer require a builder with room structures");
        };

        for room in rooms.iter() {
            for y in room.y1 + 1..=room.y2 {
                for x in room.x1 + 1..=room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx < build_data.map.width as usize * build_data.map.height as usize {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }
    }
}
