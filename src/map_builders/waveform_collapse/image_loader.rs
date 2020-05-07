use super::{Map, TileType};
use rltk::rex::XpFile;

/// Loads a RexPaint file, and converts it into our map format
pub fn load_rex_map(depth: i32, xp_file: &XpFile) -> Map {
    let mut map = Map::new(depth);

    for layer in &xp_file.layers {
        for y in 0..layer.height {
            for x in 0..layer.width {
                let cell = layer.get(x, y).unwrap();
                if x < map.width as usize && y < map.height as usize {
                    let idx = map.xy_idx(x as i32, y as i32);
                    match cell.ch {
                        32 => map.tiles[idx] = TileType::Floor,
                        35 => map.tiles[idx] = TileType::Wall, // #
                        _ => {}
                    }
                }
            }
        }
    }

    map
}
