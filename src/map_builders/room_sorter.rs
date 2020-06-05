use super::{BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;

pub struct RoomSorter {}

impl MetaMapBuilder for RoomSorter {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    pub fn new() -> Box<Self> {
        Box::new(RoomSorter {})
    }

    fn sorter(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data
            .rooms
            .as_mut()
            .unwrap()
            .sort_by(|a, b| a.x1.cmp(&b.x1));
    }
}
