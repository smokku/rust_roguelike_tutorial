use super::{Map, TileType};
use std::cmp::{max, min};

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx] = TileType::Floor;
        }
    }
}

pub fn draw_corridor(map: &mut Map, x1: i32, y1: i32, x2: i32, y2: i32) {
    let mut x = x1;
    let mut y = y1;

    while x != x2 || y != y2 {
        if x < x2 {
            x += 1;
        } else if x > x2 {
            x -= 1;
        } else if y < y2 {
            y += 1;
        } else if y > y2 {
            y -= 1;
        }

        let idx = map.xy_idx(x, y);
        map.tiles[idx] = TileType::Floor;
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
    SemiBoth,
    Both,
}

pub fn paint(map: &mut Map, mode: Symmetry, brush_size: i32, x: i32, y: i32) {
    match mode {
        Symmetry::None => apply_paint(map, brush_size, x, y),
        Symmetry::Horizontal => {
            let center_x = map.width / 2;
            let dist_x = i32::abs(center_x - x);
            if dist_x == 0 {
                apply_paint(map, brush_size, x, y);
            } else {
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
            }
        }
        Symmetry::Vertical => {
            let center_y = map.height / 2;
            let dist_y = i32::abs(center_y - y);
            if dist_y == 0 {
                apply_paint(map, brush_size, x, y);
            } else {
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
        Symmetry::SemiBoth => {
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            let dist_x = i32::abs(center_x - x);
            let dist_y = i32::abs(center_y - y);
            if dist_x == 0 && dist_y == 0 {
                apply_paint(map, brush_size, x, y);
            } else {
                // This gives only 3 symmetric points, as 2 of the points
                // will get the same and painted twice
                apply_paint(map, brush_size, center_x + dist_x, y);
                apply_paint(map, brush_size, center_x - dist_x, y);
                apply_paint(map, brush_size, x, center_y + dist_y);
                apply_paint(map, brush_size, x, center_y - dist_y);
            }
        }
        Symmetry::Both => {
            let center_x = map.width / 2;
            let center_y = map.height / 2;
            let dist_x = i32::abs(center_x - x);
            let dist_y = i32::abs(center_y - y);
            if dist_x == 0 && dist_y == 0 {
                apply_paint(map, brush_size, x, y);
            } else {
                apply_paint(map, brush_size, center_x + dist_x, center_y + dist_y);
                apply_paint(map, brush_size, center_x - dist_x, center_y - dist_y);
                apply_paint(map, brush_size, center_x - dist_x, center_y + dist_y);
                apply_paint(map, brush_size, center_x + dist_x, center_y - dist_y);
            }
        }
    }
}

pub fn apply_paint(map: &mut Map, brush_size: i32, mut x: i32, mut y: i32) {
    match brush_size {
        1 => {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
        brush_size => {
            let half_brush_size = brush_size / 2;
            x -= half_brush_size;
            y -= half_brush_size;
            for brush_y in y..y + brush_size {
                for brush_x in x..x + brush_size {
                    if brush_x > 1
                        && brush_x < map.width - 1
                        && brush_y > 1
                        && brush_y < map.height - 1
                    {
                        let idx = map.xy_idx(brush_x, brush_y);
                        map.tiles[idx] = TileType::Floor;
                    }
                }
            }
        }
    }
}
