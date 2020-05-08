use super::{Map, TileType};
use std::collections::HashSet;

pub fn build_pattern(
    map: &Map,
    chunk_size: i32,
    include_flipping: bool,
    dedupe: bool,
) -> Vec<Vec<TileType>> {
    let chunks_x = map.width / chunk_size;
    let chunks_y = map.height / chunk_size;
    let mut patterns = Vec::new();

    for cy in 0..chunks_y {
        for cx in 0..chunks_x {
            // Normal orientation
            let mut pattern = Vec::new();
            let start_x = cx * chunk_size;
            let end_x = start_x + chunk_size;
            let start_y = cy * chunk_size;
            let end_y = start_y + chunk_size;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let idx = map.xy_idx(x, y);
                    pattern.push(map.tiles[idx]);
                }
            }
            patterns.push(pattern);

            if include_flipping {
                // Flip horizontal
                pattern = Vec::new();
                for y in start_y..end_y {
                    for x in (start_x..end_x).rev() {
                        let idx = map.xy_idx(x, y);
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip vertical
                pattern = Vec::new();
                for y in (start_y..end_y).rev() {
                    for x in start_x..end_x {
                        let idx = map.xy_idx(x, y);
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);

                // Flip both
                pattern = Vec::new();
                for y in (start_y..end_y).rev() {
                    for x in (start_x..end_x).rev() {
                        let idx = map.xy_idx(x, y);
                        pattern.push(map.tiles[idx]);
                    }
                }
                patterns.push(pattern);
            }
        }
    }

    // Dedupe
    if dedupe {
        rltk::console::log(format!(
            "Pre de-duplication, there are {} patterns",
            patterns.len()
        ));
        let set: HashSet<Vec<TileType>> = patterns.drain(..).collect();
        patterns.extend(set);
        rltk::console::log(format!("There are {} patterns", patterns.len()));
    }

    patterns
}

pub fn render_pattern_to_map(
    map: &mut Map,
    pattern: &Vec<TileType>,
    chunk_size: i32,
    x: i32,
    y: i32,
) {
    let mut i = 0usize;
    for tile_y in 0..chunk_size {
        for tile_x in 0..chunk_size {
            let idx = map.xy_idx(x + tile_x, y + tile_y);
            map.tiles[idx] = pattern[i];
            map.visible_tiles[idx] = true;
            i += 1;
        }
    }
}
