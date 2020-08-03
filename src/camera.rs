use super::{map::tile_glyph, Hidden, Map, Position, Renderable};
use legion::prelude::*;
use rltk::{Point, Rltk, RGB};

const SHOW_BOUNDARIES: bool = true;
const MAP_OFFSET_X: i32 = 1;
const MAP_OFFSET_Y: i32 = 1;

pub fn get_screen_bounds(resources: &Resources, _ctx: &Rltk) -> (i32, i32, i32, i32) {
    let player_pos = resources.get::<Point>().unwrap();
    let (x_chars, y_chars) = (48, 44);

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
    let mut y = MAP_OFFSET_Y;
    for ty in min_y..max_y {
        let mut x = MAP_OFFSET_X;
        for tx in min_x..max_x {
            if tx >= 0 && tx < map.width && ty >= 0 && ty < map.height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
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
            if entity_screen_x >= 0
                && entity_screen_x < map.width
                && entity_screen_y >= 0
                && entity_screen_y < map.height
            {
                ctx.set(
                    entity_screen_x + MAP_OFFSET_X,
                    entity_screen_y + MAP_OFFSET_Y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
            }
        }
    }
}

pub fn render_debug_map(map: &Map, ctx: &mut Rltk) {
    let player_pos = Point::new(map.width / 2, map.height / 2);
    let (x_chars, y_chars) = ctx.get_char_size();

    let center_x = (x_chars / 2) as i32;
    let center_y = (y_chars / 2) as i32;

    let min_x = player_pos.x - center_x;
    let max_x = min_x + x_chars as i32;
    let min_y = player_pos.y - center_y;
    let max_y = min_y + y_chars as i32;

    let map_width = map.width - 1;
    let map_height = map.height - 1;

    let mut y = 0;
    for ty in min_y..max_y {
        let mut x = 0;
        for tx in min_x..max_x {
            if tx >= 0 && tx < map_width && ty >= 0 && ty < map_height {
                let idx = map.xy_idx(tx, ty);
                if map.revealed_tiles[idx] {
                    let (glyph, fg, bg) = tile_glyph(idx, &*map);
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
}
