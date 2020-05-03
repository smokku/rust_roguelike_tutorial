use super::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, spawner, Map,
    MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct MazeBuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for MazeBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources) {
        for (_id, area) in self.noise_areas.iter() {
            spawner::spawn_region(world, resources, area, self.map.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl MazeBuilder {
    pub fn new(depth: i32) -> Self {
        MazeBuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Maze gen
        let mut maze = Grid::new(
            (self.map.width / 2) - 2,
            (self.map.height / 2) - 2,
            &mut rng,
        );
        maze.generate_maze(self);

        // Set starting point
        self.starting_position = Position { x: 2, y: 2 };

        // Find all tiles we can reach from the starting point
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a nose map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}

/* Maze code taken under MIT from https://github.com/cyucelen/mazeGenerator/ */

const TOP: usize = 0;
const RIGHT: usize = 1;
const BOTTOM: usize = 2;
const LEFT: usize = 3;

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: [bool; 4],
    visited: bool,
}

impl Cell {
    pub fn new(row: i32, column: i32) -> Self {
        Cell {
            row,
            column,
            walls: [true, true, true, true],
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls[LEFT] = false;
            next.walls[RIGHT] = false;
        } else if x == -1 {
            self.walls[RIGHT] = false;
            next.walls[LEFT] = false;
        } else if y == 1 {
            self.walls[TOP] = false;
            next.walls[BOTTOM] = false;
        } else if y == -1 {
            self.walls[BOTTOM] = false;
            next.walls[TOP] = false;
        }
    }
}

struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &'a mut RandomNumberGenerator) -> Self {
        let mut grid = Grid {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };

        for row in 0..height {
            for col in 0..width {
                grid.cells.push(Cell::new(row, col));
            }
        }

        grid
    }

    fn calculate_index(&self, row: i32, col: i32) -> i32 {
        if row < 0 || col < 0 || col >= self.width || row >= self.height {
            -1
        } else {
            col + row * self.width
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors = Vec::new();

        let current_row = self.cells[self.current].row;
        let current_col = self.cells[self.current].column;

        let neighbor_indices = [
            self.calculate_index(current_row - 1, current_col),
            self.calculate_index(current_row, current_col + 1),
            self.calculate_index(current_row + 1, current_col),
            self.calculate_index(current_row, current_col - 1),
        ];

        for i in neighbor_indices.iter() {
            if *i != -1 && !self.cells[*i as usize].visited {
                neighbors.push(*i as usize);
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        match neighbors.len() {
            0 => None,
            1 => Some(neighbors[0]),
            len => Some(neighbors[(self.rng.roll_dice(1, len as i32) - 1) as usize]),
        }
    }

    fn generate_maze(&mut self, generator: &mut MazeBuilder) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;
            let next = self.find_next_cell();

            match next {
                Some(next) => {
                    self.backtrace.push(self.current);
                    let (lower_part, higher_part) =
                        self.cells.split_at_mut(std::cmp::max(self.current, next));
                    let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                    let cell2 = &mut higher_part[0];
                    cell1.remove_walls(cell2);
                    self.current = next;
                }
                None => {
                    if let Some(back) = self.backtrace.pop() {
                        self.current = back;
                    } else {
                        break;
                    }
                }
            }

            if i % 50 == 0 {
                self.copy_to_map(&mut generator.map);
                generator.take_snapshot();
            }
            i += 1;
        }
    }

    fn copy_to_map(&self, map: &mut Map) {
        // Clear the map
        for i in map.tiles.iter_mut() {
            *i = TileType::Wall;
        }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls[TOP] {
                map.tiles[idx - map.width as usize] = TileType::Floor;
            }
            if !cell.walls[RIGHT] {
                map.tiles[idx + 1] = TileType::Floor;
            }
            if !cell.walls[BOTTOM] {
                map.tiles[idx + map.width as usize] = TileType::Floor;
            }
            if !cell.walls[LEFT] {
                map.tiles[idx - 1] = TileType::Floor;
            }
        }
    }
}
