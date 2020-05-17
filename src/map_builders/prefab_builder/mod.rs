use super::{
    get_central_starting_position, remove_unreachable_areas_returning_most_distant, spawner, Map,
    MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

mod prefab_levels;
mod prefab_sections;

#[derive(Clone, PartialEq)]
pub enum PrefabMode {
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    Sectional {
        section: prefab_sections::PrefabSection,
    },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    mode: PrefabMode,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>,
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

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
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

impl PrefabBuilder {
    pub fn new(depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> Self {
        PrefabBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            mode: PrefabMode::Sectional {
                section: prefab_sections::UNDERGROUND_FORT,
            },
            previous_builder,
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            // Set a central starting point
            self.starting_position = get_central_starting_position(&self.map);

            // Find all tiles we can reach from the starting point
            let start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
            let exit_tile =
                remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    fn char_to_map(&mut self, ch: char, idx: usize) {
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '>' => self.map.tiles[idx] = TileType::DownStairs,

            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                self.starting_position = Position {
                    x: x as i32,
                    y: y as i32,
                };
            }

            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawn_list.push((idx, "Health Potion".to_string()));
            }

            c => {
                rltk::console::log(format!("Unknown glyph loading map: {}", c));
            }
        }
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template: &str, width: usize) -> Vec<char> {
        template
            .lines()
            .map(|line| format!("{: <width$}", line, width = width))
            .collect::<Vec<_>>()
            .concat()
            .chars()
            .collect()
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        let string_vec = PrefabBuilder::read_ascii_to_vec(level.template, level.width);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map().clone();
        self.take_snapshot();

        use prefab_sections::*;

        let string_vec = PrefabBuilder::read_ascii_to_vec(section.template, section.width);

        // Place the new section
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (self.map.width - section.width as i32) / 2,
            HorizontalPlacement::Right => self.map.width - 1 - section.width as i32,
        };
        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (self.map.height - section.height as i32) / 2,
            VerticalPlacement::Bottom => self.map.height - 1 - section.height as i32,
        };

        let mut i = 0;
        for ty in chunk_y..chunk_y + section.height as i32 {
            for tx in chunk_x..chunk_x + section.width as i32 {
                if tx < self.map.width && ty < self.map.height {
                    let idx = self.map.xy_idx(tx, ty);
                    self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }
}
