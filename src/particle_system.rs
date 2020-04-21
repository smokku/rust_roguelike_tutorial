use super::{ParticleLifetime, Position, Renderable, Rltk};
use legion::prelude::*;
use rltk::{FontCharType, RGB};

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

struct ParticleRequest {
    x: i32,
    y: i32,
    fg: RGB,
    bg: RGB,
    glyph: FontCharType,
    lifetime: f32,
}

pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> Self {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        x: i32,
        y: i32,
        fg: RGB,
        bg: RGB,
        glyph: FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            x,
            y,
            fg,
            bg,
            glyph,
            lifetime,
        })
    }
}

pub fn particle_spawn() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("particle_spawn")
        .write_resource::<ParticleBuilder>()
        .build(|command_buffer, _, particle_builder, _| {
            let particles: Vec<(Position, Renderable, ParticleLifetime)> = particle_builder
                .requests
                .iter()
                .map(|new_particle| {
                    (
                        Position {
                            x: new_particle.x,
                            y: new_particle.y,
                        },
                        Renderable {
                            fg: new_particle.fg,
                            bg: new_particle.bg,
                            glyph: new_particle.glyph,
                            render_order: 0,
                        },
                        ParticleLifetime {
                            lifetime_ms: new_particle.lifetime,
                        },
                    )
                })
                .collect();
            particle_builder.requests.clear();
            command_buffer.insert((), particles);
        })
}
