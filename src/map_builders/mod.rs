use super::{Map, Rect, TileType};

mod common;
use common::*;

mod simple_map;
use simple_map::SimpleMapBuilder;

trait MapBuilder {
    fn build(depth: i32) -> Map;
}

pub fn build_random_map(depth: i32) -> Map {
    SimpleMapBuilder::build(depth)
}
