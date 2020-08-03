use super::{
    AreaStartingPosition, BuilderChain, CellularAutomataBuilder, CullUnreachable, DistantExit,
    VoronoiSpawning, XStart, YStart,
};
use rltk::RandomNumberGenerator;

pub fn forest_builder(
    depth: i32,
    width: i32,
    height: i32,
    _rng: &mut RandomNumberGenerator,
) -> BuilderChain {
    let mut chain = BuilderChain::new(depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::LEFT, YStart::CENTER));

    // Setup an exit and spawn mobs
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain
}
