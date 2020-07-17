use super::Renderable;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Mob {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub blocks_tile: bool,
    pub stats: MobStats,
    pub vision_range: i32,
    pub ai: String,
    pub quips: Option<Vec<String>>,
    pub attributes: MobAttributes,
}

#[derive(Deserialize, Debug)]
pub struct MobStats {
    pub max_hp: i32,
    pub hp: i32,
    pub power: i32,
    pub defense: i32,
}

#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub might: Option<i32>,
    pub fitness: Option<i32>,
    pub quickness: Option<i32>,
    pub intelligence: Option<i32>,
}
