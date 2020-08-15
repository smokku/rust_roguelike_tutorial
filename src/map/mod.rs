use legion::prelude::*;
use rltk::{Algorithm2D, BaseMap, Point, SmallVec};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use type_uuid::TypeUuid;

mod tile_type;
pub use tile_type::{tile_cost, tile_opaque, tile_walkable, TileType};
mod fov;
pub use fov::field_of_view;
mod astar;
pub use astar::a_star_search;
mod themes;
pub use themes::tile_glyph;
pub mod dungeon;

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
    pub name: String,

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
            self.blocked[i] = !tile_walkable(*tile);
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Generates an empty map, consisting entirely of solid walls
    pub fn new<S: ToString>(depth: i32, width: i32, height: i32, name: S) -> Self {
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
            name: name.to_string(),
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        idx >= self.tiles.len() || tile_opaque(self.tiles[idx]) || self.view_blocked.contains(&idx)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
        let tt = self.tiles[idx];

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, tile_cost(tt)));
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, tile_cost(tt)));
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, tile_cost(tt)));
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, tile_cost(tt)));
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - 1 - w, tile_cost(tt) * 1.45));
        };
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx + 1 - w, tile_cost(tt) * 1.45));
        };
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx - 1 + w, tile_cost(tt) * 1.45));
        };
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + 1 + w, tile_cost(tt) * 1.45));
        };

        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
