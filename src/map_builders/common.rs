use super::{Map, Position, Rect, TileType};
use rltk::RandomNumberGenerator;
use std::cmp::{max, min};
use std::collections::HashMap;

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }
}

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

/// Searches a map, removes unreachable areas and returns the most distant tile.
pub fn remove_unreachable_areas_returning_most_distant(map: &mut Map, start_idx: usize) -> usize {
    map.populate_blocked();

    // Find all the tiles we can reach from the starting point
    let map_starts: Vec<usize> = vec![start_idx];
    let dijkstra_map = rltk::DijkstraMap::new(map.width, map.height, &map_starts, map, 1000.0);
    let mut farthest_tile = 0;
    let mut farthest_tile_distance = 0.0f32;
    for (i, tile) in map.tiles.iter_mut().enumerate() {
        if *tile == TileType::Floor {
            let distance_to_start = dijkstra_map.map[i];
            if distance_to_start == std::f32::MAX {
                // We can't get to this tile, so we'll make it a wall
                *tile = TileType::Wall;
            } else {
                // if it is further away than our current exit candidate, move the exit
                if distance_to_start > farthest_tile_distance {
                    farthest_tile = i;
                    farthest_tile_distance = distance_to_start;
                }
            }
        }
    }

    // Return farthest tile
    farthest_tile
}

/// Generates a Voronoi/cellular noise map of a region, and divides it into spawn regions.
pub fn generate_voronoi_spawn_regions(
    map: &Map,
    rng: &mut RandomNumberGenerator,
) -> HashMap<i32, Vec<usize>> {
    let mut noise_areas: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65526) as u64);
    noise.set_noise_type(rltk::NoiseType::Cellular);
    noise.set_frequency(0.08);
    noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] == TileType::Floor {
                let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                let cell_value = cell_value_f as i32;

                if noise_areas.contains_key(&cell_value) {
                    noise_areas.get_mut(&cell_value).unwrap().push(idx);
                } else {
                    noise_areas.insert(cell_value, vec![idx]);
                }
            }
        }
    }

    noise_areas
}

/// Find a starting point by starting at the center and moving left, searching for a floor tile.
pub fn get_central_starting_position(map: &Map) -> Position {
    let mut starting_position = Position {
        x: map.width / 2,
        y: map.height / 2,
    };
    let mut start_idx;
    while {
        start_idx = map.xy_idx(starting_position.x, starting_position.y);
        map.tiles[start_idx] != TileType::Floor
    } {
        starting_position.x -= 1;
        if starting_position.x < 0 {
            panic!("Cannot find starting position");
        }
    }

    starting_position
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Symmetry {
    None,
    Horizontal,
    Vertical,
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
