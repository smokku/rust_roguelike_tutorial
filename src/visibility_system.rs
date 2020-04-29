use super::{gamelog::GameLog, Hidden, Map, Name, Player, Position, Viewshed};
use legion::prelude::*;
use rltk::{field_of_view, Point};
use std::collections::HashSet;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("visibility_system")
        .write_resource::<Map>()
        .write_resource::<rltk::RandomNumberGenerator>()
        .write_resource::<GameLog>()
        .with_query(<(Write<Viewshed>, Read<Position>)>::query())
        .read_component::<Player>()
        .read_component::<Name>()
        .build(|command_buffer, world, (map, rng, log), query| {
            let mut seen_tiles: HashSet<usize> = HashSet::new();
            for chunk in query.iter_chunks_mut(world) {
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
                            seen_tiles.insert(idx);
                        }
                    }
                }
            }

            for idx in seen_tiles.iter() {
                // Chance to reveal hidden things
                for e in map.tile_content[*idx].iter() {
                    if let Some(_hidden) = world.get_tag::<Hidden>(*e) {
                        if rng.roll_dice(1, 24) == 1 {
                            if let Some(name) = world.get_component::<Name>(*e) {
                                log.entries.push(format!("You spotted a {}.", &name.name));
                            }
                            command_buffer.remove_tag::<Hidden>(*e);
                        }
                    }
                }
            }
        })
}
