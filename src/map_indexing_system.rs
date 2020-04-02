use super::{BlocksTile, Map, Position};
use legion::prelude::*;

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("map_indexing")
        .with_query(Read::<Position>::query().filter(tag::<BlocksTile>()))
        .write_resource::<Map>()
        .build(|_, world, map, query| {
            map.populate_blocked();
            for position in query.iter(&world) {
                let idx = map.xy_idx(position.x, position.y);
                map.blocked[idx] = true;
            }
        })
}
