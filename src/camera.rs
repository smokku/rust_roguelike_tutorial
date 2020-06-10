use super::{Hidden, Map, Position, Renderable, TileType};
use legion::prelude::*;
use rltk::{Point, Rltk, RGB};

const SHOW_BOUNDARIES: bool = true;

pub fn get_screen_bounds(resources: &Resources, ctx: &Rltk) -> (i32, i32, i32, i32) {
    let player_pos = resources.get::<Point>().unwrap();
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    (min_x, max_x, min_y, max_y)
}

pub fn render_camera(world: &World, resources: &Resources, ctx: &mut Rltk) {
    let map = resources.get::<Map>().unwrap();
    let (min_x, max_x, min_y, max_y) = get_screen_bounds(resources, ctx);

    // Draw the Map
    let map_width = map.width - 1;
    let map_height = map.height - 1;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx > 0 && tx < map_width && ty > 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = get_tile_glyph(idx, &*map);
                    ctx.set(x, y, fg, bg, glyph);
                }
            } else if SHOW_BOUNDARIES {
                ctx.set(
                    x,
                    y,
                    RGB::named(rltk::GRAY),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('·'),
                );
            }
            x += 1;
        }
        y += 1;
    }

    // Draw Renderable entities
    let query = <(Read<Position>, Read<Renderable>)>::query().filter(!tag::<Hidden>());
    let mut data = query.iter(world).collect::<Vec<_>>();
    data.sort_by(|a, b| b.1.render_order.cmp(&a.1.render_order));
    for (pos, render) in data.iter() {
        let idx = map.xy_idx(pos.x, pos.y);
        if map.visible_tiles[idx] {
            let entity_screen_x = pos.x - min_x;
            let entity_screen_y = pos.y - min_y;
            if entity_screen_x > 0
                && entity_screen_x < map_width
                && entity_screen_y > 0
                && entity_screen_y < map_height
            {
                ctx.set(
                    entity_screen_x,
                    entity_screen_y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
            }
        }
    }

    fn get_tile_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
        let glyph;
        let mut fg;
        let mut bg = RGB::from_f32(0., 0., 0.);

        match map.tiles[idx] {
            TileType::Floor => {
                glyph = rltk::to_cp437('.');
                fg = RGB::from_f32(0.0, 0.5, 0.5);
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
        }
        if map.visible_tiles[idx] {
            if map.bloodstains.contains(&idx) {
                bg = RGB::from_f32(0.75, 0.0, 0.0);
            }
        } else {
            fg = fg.to_greyscale()
        }

        (glyph, fg, bg)
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

    fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
        if x < 0 || x >= map.width || y < 0 || y >= map.height {
            return false;
        }
        let idx = map.xy_idx(x, y);
        map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
    }
}
