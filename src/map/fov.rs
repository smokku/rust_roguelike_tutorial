use super::Map;
use num_rational::Ratio;
use rltk::{Algorithm2D, BaseMap, BresenhamCircleNoDiag, Point};

pub fn field_of_view(origin: Point, range: usize, map: &Map) -> Vec<Point> {
    let mut visible_tiles = compute_fov(origin, range, map);
    // build a vector of strides of FoV circle rows
    let mut ranges: Vec<(i32, i32)> = vec![(origin.x, origin.x); range * 2 + 1];
    BresenhamCircleNoDiag::new(origin, range as i32).for_each(|point| {
        let idx = point.y - (origin.y - range as i32);
        if idx >= 0 {
            if ranges[idx as usize].0 > point.x {
                ranges[idx as usize].0 = point.x
            }
            if ranges[idx as usize].1 < point.x {
                ranges[idx as usize].1 = point.x
            }
        }
    });
    // Retail tiles in visibility circle
    visible_tiles.retain(|point| {
        if point.x < 0 || point.x >= map.width || point.y < 0 || point.y >= map.height {
            return false;
        }

        let idx = point.y - (origin.y - range as i32);
        if idx < 0 {
            return false;
        }

        let (min, max) = ranges[idx as usize];
        min <= point.x && point.x <= max
    });
    visible_tiles
}

// Symmetric Shadowcasting FoV implementation
// Based on https://www.albertford.com/shadowcasting/

fn scan(
    row: &mut Row,
    quadrant: &Quadrant,
    range: usize,
    map: &Map,
    visible_tiles: &mut Vec<Point>,
) {
    if row.depth > range {
        return;
    }

    let mut prev_tile = None;
    let (depth, min_col, max_col) = row.tiles();
    for col in min_col..=max_col {
        let tile = (depth as i32, col);
        if is_wall(Some(tile), quadrant, map) || is_symmetric(&row, tile) {
            reveal(tile, quadrant, visible_tiles);
        }
        if is_wall(prev_tile, quadrant, map) && is_floor(Some(tile), quadrant, map) {
            row.start_slope = slope(tile);
        }
        if is_floor(prev_tile, quadrant, map) && is_wall(Some(tile), quadrant, map) {
            let mut next_row = row.next();
            next_row.end_slope = slope(tile);
            scan(&mut next_row, quadrant, range, map, visible_tiles);
        }
        prev_tile = Some(tile);
    }
    if is_floor(prev_tile, quadrant, map) {
        let mut next_row = row.next();
        scan(&mut next_row, quadrant, range, map, visible_tiles);
    }
}
/*
fn scan_iterative(row: Row) {
    let mut rows = vec![row];
    while !rows.isEmpty() {
        let row = rows.pop();
        let prev_tile = None;
        for tile in row.tiles() {
            if is_wall(tile) || is_symmetric(row, tile) {
                reveal(tile);
            }
            if is_wall(prev_tile) && is_floor(tile) {
                row.start_slope = slope(tile);
            }
            if is_floor(prev_tile) && is_wall(tile) {
                next_row = row.next();
                next_row.end_slope = slope(tile);
                rows.append(next_row);
            }
            prev_tile = tile;
        }
        if is_floor(prev_tile) {
            rows.append(row.next());
        }
    }
}
*/

#[inline]
fn reveal(tile: (i32, i32), quadrant: &Quadrant, visible_tiles: &mut Vec<Point>) {
    let (x, y) = quadrant.transform(tile);
    visible_tiles.push(Point::new(x, y));
}

#[inline]
fn is_wall(tile: Option<(i32, i32)>, quadrant: &Quadrant, map: &Map) -> bool {
    match tile {
        None => false,
        Some(tile) => {
            let (x, y) = quadrant.transform(tile);
            if x < 0 || x >= map.width || y < 0 || y >= map.height {
                return true;
            };
            map.is_opaque(map.point2d_to_index(Point::new(x, y)))
        }
    }
}

#[inline]
fn is_floor(tile: Option<(i32, i32)>, quadrant: &Quadrant, map: &Map) -> bool {
    match tile {
        None => false,
        Some(tile) => {
            let (x, y) = quadrant.transform(tile);
            if x < 0 || x >= map.width || y < 0 || y >= map.height {
                return false;
            };
            !map.is_opaque(map.point2d_to_index(Point::new(x, y)))
        }
    }
}

fn compute_fov(origin: Point, range: usize, map: &Map) -> Vec<Point> {
    let mut visible_tiles = Vec::new();

    // mark_visible(origin);
    visible_tiles.push(origin);

    for i in &[
        Cardinal::North,
        Cardinal::East,
        Cardinal::South,
        Cardinal::West,
    ] {
        let quadrant = Quadrant::new(*i, origin);

        let mut first_row = Row::new(1, Ratio::from_integer(-1), Ratio::from_integer(1));
        scan(&mut first_row, &quadrant, range, map, &mut visible_tiles);
    }

    visible_tiles
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cardinal {
    North,
    East,
    South,
    West,
}

struct Quadrant {
    cardinal: Cardinal,
    ox: i32,
    oy: i32,
}

impl Quadrant {
    pub fn new(cardinal: Cardinal, origin: Point) -> Self {
        Self {
            cardinal,
            ox: origin.x,
            oy: origin.y,
        }
    }

    fn transform(&self, tile: (i32, i32)) -> (i32, i32) {
        let (row, col) = tile;
        match self.cardinal {
            Cardinal::North => (self.ox + col, self.oy - row),
            Cardinal::South => (self.ox + col, self.oy + row),
            Cardinal::East => (self.ox + row, self.oy + col),
            Cardinal::West => (self.ox - row, self.oy + col),
        }
    }
}

struct Row {
    depth: usize,
    start_slope: Ratio<i32>,
    end_slope: Ratio<i32>,
}

impl Row {
    pub fn new(depth: usize, start_slope: Ratio<i32>, end_slope: Ratio<i32>) -> Self {
        Self {
            depth,
            start_slope,
            end_slope,
        }
    }

    fn tiles(&self) -> (usize, i32, i32) {
        let min_col = round_ties_up(self.depth, self.start_slope);
        let max_col = round_ties_down(self.depth, self.end_slope);
        (self.depth, min_col, max_col)
    }

    fn next(&self) -> Row {
        Row::new(self.depth + 1, self.start_slope, self.end_slope)
    }
}

#[inline]
fn slope(tile: (i32, i32)) -> Ratio<i32> {
    let (row_depth, col) = tile;
    Ratio::new(2 * col - 1, 2 * row_depth)
}

fn is_symmetric(row: &Row, tile: (i32, i32)) -> bool {
    let (_row_depth, col) = tile;
    Ratio::from_integer(col) >= Ratio::from_integer(row.depth as i32) * row.start_slope
        && Ratio::from_integer(col) <= Ratio::from_integer(row.depth as i32) * row.end_slope
}
fn round_ties_up(d: usize, n: Ratio<i32>) -> i32 {
    let multiplied = Ratio::from_integer(d as i32) * n;
    let ratio = *multiplied.numer() as f32 / *multiplied.denom() as f32;
    f32::floor(ratio + 0.5) as i32
}
fn round_ties_down(d: usize, n: Ratio<i32>) -> i32 {
    let multiplied = Ratio::from_integer(d as i32) * n;
    let ratio = *multiplied.numer() as f32 / *multiplied.denom() as f32;
    f32::ceil(ratio - 0.5) as i32
}
