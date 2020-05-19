use super::{draw_corridor, BuilderMap, InitialMapBuilder, Rect, TileType};
use rltk::RandomNumberGenerator;

const MIN_ROOM_SIZE: i32 = 8;

pub struct BspInteriorBuilder {
    rects: Vec<Rect>,
}

impl InitialMapBuilder for BspInteriorBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl BspInteriorBuilder {
    pub fn new() -> Box<Self> {
        Box::new(BspInteriorBuilder { rects: Vec::new() })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut rooms: Vec<Rect> = Vec::new();
        self.rects.clear();
        self.rects.push(Rect::new(
            1,
            1,
            build_data.map.width - 2,
            build_data.map.height - 2,
        )); // Start with a single map-sized rectangle
        self.partition_rects(rng); // Divide the first room

        for r in self.rects.clone().iter() {
            let room = *r;
            rooms.push(room);
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    let idx = build_data.map.xy_idx(x, y);
                    if idx > 0
                        && idx < ((build_data.map.width * build_data.map.height) - 1) as usize
                    {
                        build_data.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            build_data.take_snapshot();
        }

        // Now we want corridors
        for i in 0..rooms.len() - 1 {
            let room = rooms[i];
            let next_room = rooms[i + 1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2)) - 1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2)) - 1);
            let end_x =
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2)) - 1);
            let end_y =
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2)) - 1);
            draw_corridor(&mut build_data.map, start_x, start_y, end_x, end_y);
            build_data.take_snapshot();
        }

        build_data.rooms = Some(rooms);
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
