use legion::prelude::*;
use rltk::{Algorithm2D, BaseMap, Point, SmallVec};
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
