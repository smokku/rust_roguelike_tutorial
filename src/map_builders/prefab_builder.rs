use super::{
    get_central_starting_position, remove_unreachable_areas_returning_most_distant, spawner, Map,
    MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

#[derive(Clone, Debug, PartialEq)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    mode: PrefabMode,
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
            mode: PrefabMode::RexLevel {
                template: "../resources/wfc-demo1.xp",
            },
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
        }

        // Set a central starting point
        self.starting_position = get_central_starting_position(&self.map);
        self.take_snapshot();
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        match (cell.ch as u8) as char {
                            ' ' => self.map.tiles[idx] = TileType::Floor,
                            '#' => self.map.tiles[idx] = TileType::Wall,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
