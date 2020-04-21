use super::{ParticleLifetime, Rltk};
use legion::prelude::*;

pub fn cull_dead_particles(world: &mut World, ctx: &Rltk) {
    let mut dead_particles = Vec::new();
    let query = Write::<ParticleLifetime>::query();
    for (entity, mut particle) in query.iter_entities_mut(world) {
        particle.lifetime_ms -= ctx.frame_time_ms;
        if particle.lifetime_ms < 0. {
            dead_particles.push(entity);
        }
    }

    for dead in dead_particles.iter() {
        world.delete(*dead);
    }
}
