use super::{
    generate_voronoi_spawn_regions, remove_unreachable_areas_returning_most_distant, spawner, Map,
    MapBuilder, Position, TileType, SHOW_MAPGEN_VISUALIZER,
};
use legion::prelude::*;
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DLAAlgorithm {
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DLASymmetry {
    None,
    Horizontal,
    Vertical,
    Both,
}

pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: DLASymmetry,
    floor_percent: f32,
}

impl MapBuilder for DLABuilder {
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

impl DLABuilder {
    #[allow(dead_code)]
    pub fn new(depth: i32) -> Self {
        DLABuilder {
            map: Map::new(depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 3,
            symmetry: DLASymmetry::Both,
            floor_percent: 0.25,
        }
    }
    pub fn walk_inwards(new_depth: i32) -> Self {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn walk_outwards(new_depth: i32) -> Self {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn central_attractor(new_depth: i32) -> Self {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::None,
            floor_percent: 0.25,
        }
    }

    pub fn insectoid(new_depth: i32) -> Self {
        DLABuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            history: Vec::new(),
            noise_areas: HashMap::new(),
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: DLASymmetry::Horizontal,
            floor_percent: 0.25,
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Carve a starting seed
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;
        self.take_snapshot();

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.floor_percent * total_tiles as f32) as usize;

        while {
            let floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|tile| **tile == TileType::Floor)
                .count();
            floor_tile_count < desired_floor_tiles
        } {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    while {
                        let digger_idx = self.map.xy_idx(digger_x, digger_y);
                        self.map.tiles[digger_idx] == TileType::Wall
                    } {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    self.paint(prev_x, prev_y);
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = self.starting_position.x;
                    let mut digger_y = self.starting_position.y;
                    while {
                        let digger_idx = self.map.xy_idx(digger_x, digger_y);
                        self.map.tiles[digger_idx] == TileType::Floor
                    } {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                    }
                    self.paint(digger_x, digger_y);
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham,
                        rltk::Point::new(digger_x, digger_y),
                        rltk::Point::new(self.starting_position.x, self.starting_position.y),
                    );

                    while {
                        let digger_idx = self.map.xy_idx(digger_x, digger_y);
                        self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty()
                    } {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                    }

                    self.paint(prev_x, prev_y);
                }
            }
        }

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }

    fn paint(&mut self, x: i32, y: i32) {
        match self.symmetry {
            DLASymmetry::None => self.apply_paint(x, y),
            DLASymmetry::Horizontal => {
                let center_x = self.map.width / 2;
                let dist_x = i32::abs(center_x - x);
                if dist_x == 0 {
                    self.apply_paint(x, y);
                } else {
                    self.apply_paint(center_x + dist_x, y);
                    self.apply_paint(center_x - dist_x, y);
                }
            }
            DLASymmetry::Vertical => {
                let center_y = self.map.height / 2;
                let dist_y = i32::abs(center_y - y);
                if dist_y == 0 {
                    self.apply_paint(x, y);
                } else {
                    self.apply_paint(x, center_y + dist_y);
                    self.apply_paint(x, center_y - dist_y);
                }
            }
            DLASymmetry::Both => {
                let center_x = self.map.width / 2;
                let center_y = self.map.height / 2;
                let dist_x = i32::abs(center_x - x);
                let dist_y = i32::abs(center_y - y);
                if dist_x == 0 && dist_y == 0 {
                    self.apply_paint(x, y);
                } else {
                    self.apply_paint(center_x + dist_x, center_y + dist_y);
                    self.apply_paint(center_x - dist_x, center_y - dist_y);
                    self.apply_paint(center_x - dist_x, center_y + dist_y);
                    self.apply_paint(center_x + dist_x, center_y - dist_y);
                }
            }
        }
        self.take_snapshot();
    }

    fn apply_paint(&mut self, mut x: i32, mut y: i32) {
        match self.brush_size {
            1 => {
                let idx = self.map.xy_idx(x, y);
                self.map.tiles[idx] = TileType::Floor;
            }
            brush_size => {
                let half_brush_size = brush_size / 2;
                x -= half_brush_size;
                y -= half_brush_size;
                for brush_y in y..y + brush_size {
                    for brush_x in x..x + brush_size {
                        if brush_x > 1
                            && brush_x < self.map.width - 1
                            && brush_y > 1
                            && brush_y < self.map.height - 1
                        {
                            let idx = self.map.xy_idx(brush_x, brush_y);
                            self.map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }
    }
}
