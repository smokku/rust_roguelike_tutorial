use super::{gamelog::GameLog, EntryTrigger, Hidden, Map, Name, Position};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("trigger")
        .with_query(Read::<Position>::query().filter(changed::<Position>()))
        .read_resource::<Map>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (map, log), query| {
            for (entity, pos) in query.iter_entities(world) {
                let idx = map.xy_idx(pos.x, pos.y);
                for map_entity in map.tile_content[idx].iter() {
                    let map_entity = *map_entity;
                    if entity != map_entity {
                        // Do not bother to check yourself for being a trap!
                        if let Some(_trigger) = world.get_tag::<EntryTrigger>(map_entity) {
                            // entity triggered it
                            command_buffer.remove_tag::<Hidden>(map_entity);
                            if let Some(name) = world.get_component::<Name>(map_entity) {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }
                        }
                    }
                }
            }
        })
}
