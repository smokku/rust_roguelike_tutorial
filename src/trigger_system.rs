use super::{
    gamelog::GameLog, particle_system::ParticleBuilder, EntryTrigger, Hidden, InflictsDamage, Map,
    Name, Position, SingleActivation, SufferDamage,
};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("trigger")
        .with_query(Read::<Position>::query().filter(changed::<Position>()))
        .read_resource::<Map>()
        .write_resource::<GameLog>()
        .write_resource::<ParticleBuilder>()
        .read_component::<Name>()
        .read_component::<InflictsDamage>()
        .build(
            |command_buffer, world, (map, log, particle_builder), query| {
                for (entity, pos) in query.iter_entities(world) {
                    let idx = map.xy_idx(pos.x, pos.y);
                    for map_entity in map.tile_content[idx].iter() {
                        let map_entity = *map_entity;
                        if entity != map_entity {
                            // Do not bother to check yourself for being a trap!
                            if let Some(_trigger) = world.get_tag::<EntryTrigger>(map_entity) {
                                // entity triggered it
                                command_buffer.remove_tag::<Hidden>(map_entity); // The trap is no longer hidden

                                if let Some(name) = world.get_component::<Name>(map_entity) {
                                    log.entries.push(format!("{} triggers!", &name.name));
                                }

                                // If the trap is damage inflicting, do it
                                if let Some(damage) =
                                    world.get_component::<InflictsDamage>(map_entity)
                                {
                                    particle_builder.request(
                                        pos.x,
                                        pos.y,
                                        rltk::RGB::named(rltk::ORANGE),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('‼'),
                                        200.0,
                                    );
                                    SufferDamage::new_damage(
                                        command_buffer,
                                        entity,
                                        damage.damage,
                                        false,
                                    );
                                }

                                // If it is single activation, it needs to be removed
                                if let Some(_sa) = world.get_tag::<SingleActivation>(map_entity) {
                                    command_buffer.delete(map_entity);
                                }
                            }
                        }
                    }
                }
            },
        )
}
