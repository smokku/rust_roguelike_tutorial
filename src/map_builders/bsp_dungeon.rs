use super::{
    apply_room_to_map, spawner, Map, MapBuilder, Position, Rect, Resources, TileType, World,
    SHOW_MAPGEN_VISUALIZER,
};
use rltk::RandomNumberGenerator;

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

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects
            .push(Rect::new(2, 2, self.map.width - 5, self.map.height - 5)); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.partition_rect(&first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it.
        // If its possible to squeeze a room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(&mut rng);
            let candidate = self.get_room_candidate(&rect, &mut rng);

            if self.is_possible(&candidate) {
                apply_room_to_map(&mut self.map, &candidate);
                self.rooms.push(candidate);
                self.partition_rect(&rect);
                self.take_snapshot();
            }

            n_rooms += 1;
        }
        let start = self.rooms[0].center();
        self.starting_position = Position {
            x: start.0,
            y: start.1,
        };
    }

    fn partition_rect(&mut self, rect: &Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::new(rect.x1, rect.y1, half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_width,
            rect.y1,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1 + half_height,
            rect.y1 + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_random_rect(&mut self, rng: &mut RandomNumberGenerator) -> Rect {
        self.rects[match self.rects.len() {
            1 => 0,
            len => (rng.roll_dice(1, len as i32) - 1) as usize,
        }]
    }

    fn get_room_candidate(&mut self, rect: &Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let mut result = *rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10)) - 1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10)) - 1) + 1;

        result.x1 += rng.roll_dice(1, 6) - 1;
        result.y1 += rng.roll_dice(1, 6) - 1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect: &Rect) -> bool {
        let mut expanded = *rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;

        for y in expanded.y1..=expanded.y2 {
            for x in expanded.x1..=expanded.x2 {
                if x < 1 {
                    return false;
                }
                if y < 1 {
                    return false;
                }
                if x > self.map.width - 2 {
                    return false;
                }
                if y > self.map.height - 2 {
                    return false;
                }

                let idx = self.map.xy_idx(x, y);
                if self.map.tiles[idx] != TileType::Wall {
                    return false;
                }
            }
        }
        true
    }
}
