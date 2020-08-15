use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::Floor
        | TileType::DownStairs
        | TileType::UpStairs
        | TileType::Road
        | TileType::Grass
        | TileType::ShallowWater
        | TileType::WoodFloor
        | TileType::Bridge
        | TileType::Gravel => true,
        TileType::Wall | TileType::DeepWater => false,
    }
}

pub fn tile_opaque(tt: TileType) -> bool {
    match tt {
        TileType::Wall => true,
        TileType::Floor
        | TileType::DownStairs
        | TileType::UpStairs
        | TileType::Road
        | TileType::Grass
        | TileType::ShallowWater
        | TileType::DeepWater
        | TileType::WoodFloor
        | TileType::Bridge
        | TileType::Gravel => false,
    }
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        TileType::Wall
        | TileType::Floor
        | TileType::DownStairs
        | TileType::UpStairs
        | TileType::DeepWater
        | TileType::WoodFloor
        | TileType::Bridge
        | TileType::Gravel => 1.0,
    }
}
