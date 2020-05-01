use super::{spawner, Map, Position, Rect, TileType};
use legion::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(depth: i32) -> (Map, Position);
    fn spawn(map: &mut Map, world: &mut World, resources: &mut Resources);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}

pub fn spawn(map: &mut Map, world: &mut World, resources: &mut Resources) {
    SimpleMapBuilder::spawn(map, world, resources);
}
