use legion::prelude::*;
use rltk::{Algorithm2D, BaseMap, Point, Rltk, SmallVec, RGB};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use type_uuid::TypeUuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[uuid = "09e57cda-e925-47f0-a3f6-107c86fa76bd"]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,

    #[serde(skip)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    #[inline]
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || x >= self.width || y < 0 || y >= self.height {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new(depth: i32, width: i32, height: i32) -> Self {
        let map_tile_count = (width * height) as usize;
        Map {
            tiles: vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            blocked: vec![false; map_tile_count],
            tile_content: vec![Vec::new(); map_tile_count],
            depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall || self.view_blocked.contains(&idx)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - 1 - w, 1.45))
        };
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx + 1 - w, 1.45))
        };
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx - 1 + w, 1.45))
        };
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + 1 + w, 1.45))
        };

        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    if x < 0 || x >= map.width || y < 0 || y >= map.height {
        return false;
    }
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 0 || x >= map.width || y < 0 || y >= map.height {
        return 35;
    }
    let mut mask: u8 = 0;

    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,    // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // We missed one?
    }
}

pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut x = 0;
    let mut y = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon tile type
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            let mut bg = RGB::from_f32(0., 0., 0.);
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&map, x, y);
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
            }
            if map.visible_tiles[idx] {
                if map.bloodstains.contains(&idx) {
                    bg = RGB::from_f32(0.75, 0.0, 0.0);
                }
            } else {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move the coordinates
        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}
