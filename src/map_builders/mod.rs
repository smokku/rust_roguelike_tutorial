use super::{spawner, Map, Position, Rect, TileType, SHOW_MAPGEN_VISUALIZER};
use legion::prelude::*;
use rltk::RandomNumberGenerator;

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
use voronoi::VoronoiCellBuilder;
mod waveform_collapse;
use waveform_collapse::WaveformCollapseBuilder;
mod prefab_builder;
use prefab_builder::PrefabBuilder;
mod room_based_spawner;
use room_based_spawner::RoomBasedSpawner;
mod room_based_starting_position;
use room_based_starting_position::RoomBasedStartingPosition;
mod room_based_stairs;
use room_based_stairs::RoomBasedStairs;
mod area_starting_points;
use area_starting_points::*;
mod cull_unreachable;
use cull_unreachable::CullUnreachable;
mod voronoi_spawning;
use voronoi_spawning::VoronoiSpawning;
mod distant_exit;
use distant_exit::DistantExit;
mod room_exploder;
use room_exploder::RoomExploder;

pub struct BuilderMap {
    pub map: Map,
    pub starting_position: Option<Position>,
    pub rooms: Option<Vec<Rect>>,
    pub history: Vec<Map>,
    pub spawn_list: Vec<(usize, String)>,
}

impl BuilderMap {
    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

pub struct BuilderChain {
    starter: Option<Box<dyn InitialMapBuilder>>,
    builders: Vec<Box<dyn MetaMapBuilder>>,
    pub build_data: BuilderMap,
}

impl BuilderChain {
    pub fn new(depth: i32) -> Self {
        BuilderChain {
            starter: None,
            builders: Vec::new(),
            build_data: BuilderMap {
                map: Map::new(depth),
                starting_position: None,
                rooms: None,
                history: Vec::new(),
                spawn_list: Vec::new(),
            },
        }
    }

    pub fn start_with(&mut self, starter: Box<dyn InitialMapBuilder>) {
        match self.starter {
            None => self.starter = Some(starter),
            Some(_) => panic!("You can only have one starting builder."),
        };
    }

    pub fn with(&mut self, meta_builder: Box<dyn MetaMapBuilder>) {
        self.builders.push(meta_builder);
    }

    pub fn build_map(&mut self, rng: &mut RandomNumberGenerator) {
        match &mut self.starter {
            None => panic!("Cannot run a map builder chain without a starting build system"),
            Some(starter) => {
                // Build the starting map
                starter.build_map(rng, &mut self.build_data);
            }
        }

        // Build additional layers in turn
        for metabuilder in self.builders.iter_mut() {
            metabuilder.build_map(rng, &mut self.build_data);
        }
    }

    pub fn spawn_entities(&mut self, world: &mut World) {
        for (idx, name) in self.build_data.spawn_list.iter() {
            spawner::spawn_entity(world, idx, name);
        }
    }
}

pub trait InitialMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

pub trait MetaMapBuilder {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap);
}

/// Returns initial map builder and a bool,
/// indicating whether or not we picked an algorithm that provides room data.
pub fn random_initial_builder(
    rng: &mut RandomNumberGenerator,
) -> (Box<dyn InitialMapBuilder>, bool) {
    let builder = rng.roll_dice(1, 18);
    match builder {
        1 => (BspDungeonBuilder::new(), true),
        2 => (BspInteriorBuilder::new(), true),
        3 => (CellularAutomataBuilder::new(), false),
        4 => (DrunkardsWalkBuilder::open_area(), false),
        5 => (DrunkardsWalkBuilder::open_halls(), false),
        6 => (DrunkardsWalkBuilder::winding_passages(), false),
        7 => (DrunkardsWalkBuilder::fat_passage(), false),
        8 => (DrunkardsWalkBuilder::fearful_symmetry(), false),
        9 => (MazeBuilder::new(), false),
        10 => (DLABuilder::walk_inwards(), false),
        11 => (DLABuilder::walk_outwards(), false),
        12 => (DLABuilder::central_attractor(), false),
        13 => (DLABuilder::insectoid(), false),
        14 => (VoronoiCellBuilder::pythagoras(), false),
        15 => (VoronoiCellBuilder::manhattan(), false),
        16 => (VoronoiCellBuilder::chebyshev(), false),
        17 => (
            PrefabBuilder::constant(prefab_builder::prefab_levels::WFC_POPULATED),
            false,
        ),
        _ => (SimpleMapBuilder::new(), true),
    }
}

pub fn random_builder(depth: i32, rng: &mut RandomNumberGenerator) -> BuilderChain {
    let mut builder = BuilderChain::new(depth);
    builder.start_with(BspDungeonBuilder::new());
    builder.with(RoomExploder::new());
    builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    builder.with(CullUnreachable::new());
    builder.with(VoronoiSpawning::new());
    builder.with(DistantExit::new());
    builder
    // let (random_starter, has_rooms) = random_initial_builder(rng);
    // builder.start_with(random_starter);
    // if has_rooms {
    //     builder.with(RoomBasedStartingPosition::new());
    //     builder.with(RoomBasedStairs::new());
    //     builder.with(RoomBasedSpawner::new());
    // } else {
    //     builder.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    //     builder.with(CullUnreachable::new());
    //     builder.with(VoronoiSpawning::new());
    //     builder.with(DistantExit::new());
    // }

    // if rng.roll_dice(1, 3) == 1 {
    //     builder.with(WaveformCollapseBuilder::new());
    // }

    // if rng.roll_dice(1, 20) == 1 {
    //     builder.with(PrefabBuilder::sectional(
    //         prefab_builder::prefab_sections::UNDERGROUND_FORT,
    //     ));
    // }

    // builder.with(PrefabBuilder::vaults());

    // builder
}
