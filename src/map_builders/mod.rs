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
use drunkard::{DrunkSpawnMode, DrunkardSettings, DrunkardsWalkBuilder};

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
    // let builder = rng.roll_dice(1, 4);
    // match builder {
    //     1 => Box::new(BspDungeonBuilder::new(depth)),
    //     2 => Box::new(BspInteriorBuilder::new(depth)),
    //     3 => Box::new(CellularAutomataBuilder::new(depth)),
    //     _ => Box::new(SimpleMapBuilder::new(depth)),
    // }
    Box::new(DrunkardsWalkBuilder::new(
        depth,
        DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
        },
    ))
}
