use super::{Map, TileType};
use rltk::{FontCharType, RGB};

pub fn tile_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.depth {
        // 2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map),
    };

    if map.visible_tiles[idx] {
        if map.bloodstains.contains(&idx) {
            bg = RGB::from_f32(0.75, 0.0, 0.0);
        }
    } else {
        fg = fg.to_greyscale()
    }

    (glyph, fg, bg)
}

fn get_tile_glyph_default(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let mut fg;
    let mut bg = RGB::from_f32(0.0, 0.0, 0.0);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.0, 0.5, 0.5);
        }
        TileType::WoodFloor => {
            glyph = rltk::to_cp437('░');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Wall => {
            let x = idx as i32 % map.width;
            let y = idx as i32 / map.width;
            glyph = wall_glyph(&map, x, y);
            fg = RGB::from_f32(0.0, 1.0, 0.0);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0.0, 1.0, 1.0);
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('▒');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::GRAY);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::BLUE);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            fg = RGB::named(rltk::GRAY);
        }
    }

    (glyph, fg, bg)
}

fn wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
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

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    if x < 0 || x >= map.width || y < 0 || y >= map.height {
        return false;
    }
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
