use super::{spawner, Map, MapBuilder, Position, Rect, Resources, World, SHOW_MAPGEN_VISUALIZER};

pub struct BspDungeonBuilder {
    map: Map,
    starting_position: Position,
    rooms: Vec<Rect>,
    history: Vec<Map>,
    rects: Vec<Rect>,
}

impl MapBuilder for BspDungeonBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {}

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

impl BspDungeonBuilder {
    pub fn new(depth: i32) -> BspDungeonBuilder {
        BspDungeonBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            rooms: Vec::new(),
            history: Vec::new(),
            rects: Vec::new(),
        }
    }
}
