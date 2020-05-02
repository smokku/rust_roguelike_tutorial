use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE: i32 = 8;

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
        self.build();
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

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects
            .push(Rect::new(1, 1, self.map.width - 1, self.map.height - 1)); // Start with a single map-sized rectangle
        self.partition_rects(&mut rng);

        for r in self.rects.clone().iter() {
            let room = *r;
            self.rooms.push(room);
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = self.map.xy_idx(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height) - 1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.take_snapshot();
        }

        let start = self.rooms[0].center();
        self.starting_position = Position {
            x: start.0,
            y: start.1,
        };
    }

    fn partition_rects(&mut self, rng: &mut RandomNumberGenerator) {
        if let Some(rect) = self.rects.pop() {
            // Calculate boundaries
            let width = rect.x2 - rect.x1;
            let height = rect.y2 - rect.y1;

            let split = rng.roll_dice(1, 4);
            if split <= 2 {
                // Horizontal split
                let half_width = width / 2;
                let h1 = Rect::new(rect.x1, rect.y1, half_width - 1, height);
                self.rects.push(h1);
                if half_width > MIN_ROOM_SIZE {
                    self.partition_rects(rng);
                }
                let h2 = Rect::new(rect.x1 + half_width, rect.y1, half_width, height);
                self.rects.push(h2);
                if half_width > MIN_ROOM_SIZE {
                    self.partition_rects(rng);
                }
            } else {
                // Vertical split
                let half_height = height / 2;
                let v1 = Rect::new(rect.x1, rect.y1, width, half_height - 1);
                self.rects.push(v1);
                if half_height > MIN_ROOM_SIZE {
                    self.partition_rects(rng);
                }
                let v2 = Rect::new(rect.x1, rect.y1 + half_height, width, half_height);
                self.rects.push(v2);
                if half_height > MIN_ROOM_SIZE {
                    self.partition_rects(rng);
                }
            }
        }
    }
}
