use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder, Position, TileType};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

#[derive(PartialEq, Copy, Clone)]
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
    RoomVaults,
}

pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn new() -> Box<Self> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    #[allow(dead_code)]
    pub fn rex_level(template: &'static str) -> Box<Self> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RexLevel { template },
        })
    }

    pub fn constant(level: prefab_levels::PrefabLevel) -> Box<Self> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Constant { level },
        })
    }

    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<Self> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Sectional { section },
        })
    }

    pub fn vaults() -> Box<Self> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(&template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
        }
        build_data.take_snapshot();
    }

    fn char_to_map(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        match ch {
            ' ' => build_data.map.tiles[idx] = TileType::Floor,
            '#' => build_data.map.tiles[idx] = TileType::Wall,
            '>' => build_data.map.tiles[idx] = TileType::DownStairs,

            '@' => {
                let x = idx as i32 % build_data.map.width;
                let y = idx as i32 / build_data.map.width;
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.starting_position = Some(Position {
                    x: x as i32,
                    y: y as i32,
                });
            }
            'g' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Goblin".to_string()));
            }
            'o' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Orc".to_string()));
            }
            '^' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data.spawn_list.push((idx, "Rations".to_string()));
            }
            '!' => {
                build_data.map.tiles[idx] = TileType::Floor;
                build_data
                    .spawn_list
                    .push((idx, "Health Potion".to_string()));
            }

            c => {
                rltk::console::log(format!("Unknown glyph loading map: {}", c));
            }
        }
    }

    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        self.char_to_map(cell.ch as u8 as char, idx, build_data);
                    }
                }
            }
        }
    }

    fn read_ascii_to_vec(template: &str, width: usize, height: usize) -> Vec<char> {
        let vec: Vec<char> = template
            .lines()
            .map(|line| format!("{: <width$}", line, width = width))
            .collect::<Vec<_>>()
            .concat()
            .chars()
            .collect();
        if vec.len() != width * height {
            panic!("Loaded template did not yield the expected number of characters. Got {}, expected {}.\n{:?}", vec.len(), width * height, template);
        }

        vec
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuilderMap) {
        let string_vec =
            PrefabBuilder::read_ascii_to_vec(level.template, level.width, level.height);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    if i < string_vec.len() {
                        self.char_to_map(string_vec[i], idx, build_data);
                    }
                }
                i += 1;
            }
        }
    }

    fn apply_previous_iteration<F>(
        &mut self,
        mut filter: F,
        _rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) where
        F: FnMut(i32, i32) -> bool,
    {
        let width = build_data.map.width;
        build_data.spawn_list.retain(|(idx, _name)| {
            let x = *idx as i32 % width;
            let y = *idx as i32 / width;
            filter(x, y)
        });
        build_data.take_snapshot();
    }

    fn apply_sectional(
        &mut self,
        section: &prefab_sections::PrefabSection,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        use prefab_sections::*;

        let string_vec =
            PrefabBuilder::read_ascii_to_vec(section.template, section.width, section.height);

        // Place the new section
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (build_data.map.width - section.width as i32) / 2,
            HorizontalPlacement::Right => build_data.map.width - 1 - section.width as i32,
        };
        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (build_data.map.height - section.height as i32) / 2,
            VerticalPlacement::Bottom => build_data.map.height - 1 - section.height as i32,
        };

        // Build the map
        self.apply_previous_iteration(
            |x, y| {
                x < chunk_x
                    || x > (chunk_x + section.width as i32)
                    || y < chunk_y
                    || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data,
        );

        let mut i = 0;
        for ty in chunk_y..chunk_y + section.height as i32 {
            for tx in chunk_x..chunk_x + section.width as i32 {
                if tx < build_data.map.width && ty < build_data.map.height {
                    let idx = build_data.map.xy_idx(tx, ty);
                    if i < string_vec.len() {
                        self.char_to_map(string_vec[i], idx, build_data);
                    }
                }
                i += 1;
            }
        }
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        use prefab_rooms::*;

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_x, _y| true, rng, build_data);

        // Do we want a vault at all?
        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 {
            return;
        }

        // Note that this is a place-holder and will be moved out of this function
        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, CHECKERBOARD, SILLY_SMILE];

        // Filter the vault list down to ones that are applicable to the current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| {
                build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth
            })
            .collect();

        if possible_vaults.is_empty() {
            // Bail out if there's nothing to build
            return;
        }

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles = HashSet::new();

        for _i in 0..n_vaults {
            let vault_index = if possible_vaults.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize
            };
            let vault = possible_vaults[vault_index];

            // We'll make a list of places in which the vault could fit
            let mut vault_positions = Vec::new();

            let mut idx = 0usize;
            loop {
                let x = (idx % build_data.map.width as usize) as i32;
                let y = (idx / build_data.map.width as usize) as i32;

                // Check that we won't overflow the map
                if x > 1
                    && (x + vault.width as i32) < build_data.map.width - 2
                    && y > 1
                    && (y + vault.height as i32) < build_data.map.height - 2
                {
                    let mut possible = true;
                    for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            let idx = build_data.map.xy_idx(x + tx, y + ty);
                            if build_data.map.tiles[idx] != TileType::Floor {
                                possible = false;
                                break;
                            }
                            if used_tiles.contains(&idx) {
                                possible = false;
                                break;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position { x, y });
                    }
                }

                idx += 1;
                if idx >= build_data.map.tiles.len() - 1 {
                    break;
                }
            }

            if !vault_positions.is_empty() {
                let pos_idx = if vault_positions.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize
                };
                let pos = &vault_positions[pos_idx];

                let chunk_x = pos.x;
                let chunk_y = pos.y;

                let map_width = build_data.map.width;
                build_data.spawn_list.retain(|e| {
                    let idx = e.0 as i32;
                    let x = idx % map_width;
                    let y = idx / map_width;

                    x < chunk_x
                        || x > chunk_x + vault.width as i32
                        || y < chunk_y
                        || y > chunk_y + vault.height as i32
                });

                let string_vec =
                    PrefabBuilder::read_ascii_to_vec(vault.template, vault.width, vault.height);
                let mut i = 0;
                for ty in 0..vault.height {
                    for tx in 0..vault.width {
                        let idx = build_data
                            .map
                            .xy_idx(chunk_x + tx as i32, chunk_y + ty as i32);
                        if i < string_vec.len() {
                            self.char_to_map(string_vec[i], idx, build_data);
                        }
                        used_tiles.insert(idx);
                        i += 1;
                    }
                }
                build_data.take_snapshot();

                possible_vaults.remove(vault_index);
            }
        }
    }
}
