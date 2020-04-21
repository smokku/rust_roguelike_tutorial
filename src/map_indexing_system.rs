use super::{BlocksTile, Map, Position};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("map_indexing")
        .with_query(Read::<Position>::query())
        .write_resource::<Map>()
        .build(|_, world, map, query| {
            map.populate_blocked();
            map.clear_content_index();
            for (entity, position) in query.iter_entities(&world) {
                let idx = map.xy_idx(position.x, position.y);

                // If they block, update the blocking list
                let blocker = world.get_tag::<BlocksTile>(entity);
                if let Some(_blocker) = blocker {
                    map.blocked[idx] = true;
                }

                // Push the entity to the appropriate index slot.
                // It's a Copy type, so we don't need to clone it
                // (we want to avoid moving it out of the ECS!)
                map.tile_content[idx].push(entity);
            }
        })
}
