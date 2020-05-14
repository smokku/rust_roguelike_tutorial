use super::{
    remove_unreachable_areas_returning_most_distant, spawner, Map, MapBuilder, Position, TileType,
    SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
}

impl MapBuilder for PrefabBuilder {
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

    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources) {}

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

impl PrefabBuilder {
    pub fn new(depth: i32) -> Self {
        PrefabBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
        }
    }

    fn build(&mut self) {}
}
