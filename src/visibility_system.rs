use super::{Map, Player, Position, Viewshed};
use legion::prelude::*;
use rltk::{field_of_view, Point};

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("visibility_system")
        .write_resource::<Map>()
        .with_query(<(Write<Viewshed>, Read<Position>)>::query())
        .read_component::<Player>()
        .build(|_, mut world, map, query| {
            for chunk in query.iter_chunks_mut(&mut world) {
                // Is this the players chunk?
                let p = chunk.tag::<Player>();
                let player_chunk = if let Some(_p) = p {
                    // Reset visibility
                    for t in map.visible_tiles.iter_mut() {
                        *t = false;
                    }
                    true
                } else {
                    false
                };

                let viewsheds = &mut chunk.components_mut::<Viewshed>().unwrap();
                let positions = &chunk.components::<Position>().unwrap();

                for (i, pos) in positions.iter().enumerate() {
                    let viewshed = &mut viewsheds[i];
                    if viewshed.dirty {
                        viewshed.dirty = false;
                        viewshed.visible_tiles.clear();
                        viewshed.visible_tiles =
                            field_of_view(Point::new(pos.x, pos.y), viewshed.range, &**map);
                        viewshed.visible_tiles.retain(|p| {
                            p.x >= 0 && p.x < map.width - 1 && p.y >= 0 && p.y < map.height - 1
                        });
                    }

                    // If this is the player, reveal what they can see
                    if player_chunk {
                        for vis in viewshed.visible_tiles.iter() {
                            let idx = map.xy_idx(vis.x, vis.y);
                            map.revealed_tiles[idx] = true;
                            map.visible_tiles[idx] = true;
                        }
                    }
                }
            }
        })
}
