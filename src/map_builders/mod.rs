use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use legion::prelude::*;

mod common;
use common::*;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::DLABuilder;
mod voronoi;
use voronoi::VoronoiBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;

    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    // let mut rng = rltk::RandomNumberGenerator::new();
    // let builder = rng.roll_dice(1, 14);
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(depth)),
    //     2 => Box::new(BspInteriorBuilder::new(depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(depth)),
    //     4 => Box::new(DrunkardsWalkBuilder::open_area(depth)),
    //     5 => Box::new(DrunkardsWalkBuilder::open_halls(depth)),
    //     6 => Box::new(DrunkardsWalkBuilder::winding_passage(depth)),
    //     7 => Box::new(DrunkardsWalkBuilder::fat_passage(depth)),
    //     8 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(depth)),
    //     9 => Box::new(MazeBuilder::new(depth)),
    //     10 => Box::new(DLABuilder::walk_inwards(depth)),
    //     11 => Box::new(DLABuilder::walk_outwards(depth)),
    //     12 => Box::new(DLABuilder::central_attractor(depth)),
    //     13 => Box::new(DLABuilder::insectoid(depth)),
    //     _ => Box::new(SimpleMapBuilder::new(depth)),
    // }
    Box::new(VoronoiBuilder::new(depth))
}
