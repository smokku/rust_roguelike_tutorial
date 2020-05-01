use super::{Map, Position, Rect, TileType};

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(depth: i32) -> (Map, Position);
}

pub fn build_random_map(depth: i32) -> (Map, Position) {
    SimpleMapBuilder::build(depth)
}
