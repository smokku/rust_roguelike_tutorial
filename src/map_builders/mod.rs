use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use legion::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World, resources: &mut Resources);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;

    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 2);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(depth)),
        _ => Box::new(SimpleMapBuilder::new(depth)),
    }
}
