use super::{spawner, Map, Position, Rect, TileType};
use legion::prelude::*;

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

pub trait MapBuilder {
    fn build_map(&mut self, depth: i32) -> (Map, Position);
    fn spawn_entities(&mut self, map: &mut Map, world: &mut World, resources: &mut Resources);
}

pub fn random_builder() -> Box<dyn MapBuilder> {
    // Note that until we have a second map type, this isn't even slightly random
    Box::new(SimpleMapBuilder::new())
}
