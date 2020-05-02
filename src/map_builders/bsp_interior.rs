use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

pub struct BspInteriorBuilder {
    map: Map,
    starting_position: Position,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
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
        // We should do something here
    }

    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(world, resources, room, self.map.depth);
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

impl BspInteriorBuilder {
    pub fn new(new_depth: i32) -> Self {
        BspInteriorBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
        }
    }
}
