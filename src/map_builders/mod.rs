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
    // Note that until we have a second map type, this isn't even slightly random
    Box::new(BspDungeonBuilder::new(depth))
}
