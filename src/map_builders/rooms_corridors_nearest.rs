use super::{draw_corridor, BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub struct NearestCorridors {}

impl MetaMapBuilder for NearestCorridors {
    #[allow(dead_code)]
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl NearestCorridors {
    pub fn new() -> Box<Self> {
        Box::new(NearestCorridors {})
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Nearest Corridors require a builder with room structures");
        };

        let mut connected = HashSet::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance = Vec::new();
            let room_center = room.center();
            let room_center_pt = rltk::Point::new(room_center.0, room_center.1);
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = rltk::Point::new(other_center.0, other_center.1);
                    let distance =
                        rltk::DistanceAlg::Pythagoras.distance2d(room_center_pt, other_center_pt);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();
                draw_corridor(
                    &mut build_data.map,
                    room_center.0,
                    room_center.1,
                    dest_center.0,
                    dest_center.1,
                );
                connected.insert(i);
                build_data.take_snapshot();
            }
        }
    }
}
