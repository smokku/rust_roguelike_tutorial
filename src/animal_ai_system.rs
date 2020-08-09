use super::{Carnivore, Herbivore, Item, Map, Point, Position, RunState, Viewshed, WantsToMelee};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("animal_ai")
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Herbivore>()))
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Carnivore>()))
        .write_resource::<Map>()
        .read_resource::<RunState>()
        .read_resource::<Entity>()
        .read_component::<Item>()
        .build(
            |command_buffer,
             world,
             (map, runstate, player_entity),
             (query_herbivore, query_carnivore)| unsafe {
                if **runstate != RunState::MonsterTurn {
                    return;
                }

                // Herbivores run away a lot
                for (mut viewshed, mut pos) in query_herbivore.iter_unchecked(world) {
                    let mut run_away_from = Vec::new();
                    for other_tile in viewshed.visible_tiles.iter() {
                        let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                        for other_entity in map.tile_content[view_idx].iter() {
                            // They don't run away from items
                            if world.get_component::<Item>(*other_entity).is_none() {
                                // TODO: log.debug(entity is running away from other_entity)
                                run_away_from.push(view_idx);
                            }
                        }
                    }

                    if !run_away_from.is_empty() {
                        let entity_idx = map.xy_idx(pos.x, pos.y);
                        map.populate_blocked();
                        let flee_map = rltk::DijkstraMap::new(
                            map.width as usize,
                            map.height as usize,
                            &run_away_from,
                            &**map,
                            10.0,
                        );
                        let flee_target =
                            rltk::DijkstraMap::find_highest_exit(&flee_map, entity_idx, &**map);
                        if let Some(flee_target) = flee_target {
                            if !map.blocked[flee_target] {
                                map.blocked[entity_idx] = false;
                                map.blocked[flee_target] = true;
                                viewshed.dirty = true;
                                pos.x = flee_target as i32 % map.width;
                                pos.y = flee_target as i32 / map.width;
                            }
                        }
                    }
                }

                // Carnivores just want to eat everything
                for (entity, (mut viewshed, mut pos)) in
                    query_carnivore.iter_entities_unchecked(world)
                {
                    let mut run_towards = Vec::new();
                    let mut attacked = false;
                    for other_tile in viewshed.visible_tiles.iter() {
                        let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                        for other_entity in map.tile_content[view_idx].iter() {
                            if world.get_tag::<Herbivore>(*other_entity).is_some()
                                || *other_entity == **player_entity
                            {
                                let distance = rltk::DistanceAlg::Pythagoras
                                    .distance2d(Point::new(pos.x, pos.y), *other_tile);
                                if distance < 1.5 {
                                    command_buffer.add_component(
                                        entity,
                                        WantsToMelee {
                                            target: *other_entity,
                                        },
                                    );
                                    attacked = true;
                                } else {
                                    run_towards.push(view_idx);
                                }
                            }
                        }
                    }

                    if !run_towards.is_empty() && !attacked {
                        let entity_idx = map.xy_idx(pos.x, pos.y);
                        map.populate_blocked();
                        let chase_map = rltk::DijkstraMap::new(
                            map.width as usize,
                            map.height as usize,
                            &run_towards,
                            &**map,
                            10.0,
                        );
                        let chase_target =
                            rltk::DijkstraMap::find_lowest_exit(&chase_map, entity_idx, &**map);
                        if let Some(chase_target) = chase_target {
                            if !map.blocked[chase_target] {
                                map.blocked[entity_idx] = false;
                                map.blocked[chase_target] = true;
                                viewshed.dirty = true;
                                pos.x = chase_target as i32 % map.width;
                                pos.y = chase_target as i32 / map.width;
                            }
                        }
                    }
                }
            },
        )
}
