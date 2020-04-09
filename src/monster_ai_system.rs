use super::{Confusion, Map, Monster, Position, RunState, Viewshed, WantsToMelee};
use legion::prelude::*;
use rltk::Point;

pub fn build() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("monster_ai")
        .write_resource::<Map>()
        .read_resource::<Point>()
        .read_resource::<Entity>()
        .read_resource::<RunState>()
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Monster>()))
        .write_component::<Confusion>()
        .build(
            |command_buffer, mut world, (map, player_pos, player_entity, runstate), query| {
                if **runstate != RunState::MonsterTurn {
                    return;
                }
                for (entity, (mut viewshed, mut pos)) in query.iter_entities_mut(&mut world) {
                    let mut can_act = true;

                    if let Some(mut confused) = world.get_component_mut::<Confusion>(entity) {
                        confused.turns -= 1;
                        if confused.turns < 1 {
                            command_buffer.remove_component::<Confusion>(entity);
                        }
                        can_act = false;
                    }

                    if can_act {
                        let distance = rltk::DistanceAlg::Pythagoras
                            .distance2d(Point::new(pos.x, pos.y), **player_pos);
                        if distance < 1.5 {
                            command_buffer.add_component(
                                entity,
                                WantsToMelee {
                                    target: **player_entity,
                                },
                            );
                        } else if viewshed.visible_tiles.contains(&**player_pos) {
                            // Path to the player
                            let path = rltk::a_star_search(
                                map.xy_idx(pos.x, pos.y),
                                map.xy_idx(player_pos.x, player_pos.y),
                                &**map,
                            );
                            if path.success && path.steps.len() > 1 {
                                let mut idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = false;
                                pos.x = path.steps[1] as i32 % map.width;
                                pos.y = path.steps[1] as i32 / map.width;
                                idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = true;
                                viewshed.dirty = true;
                            }
                        }
                    }
                }
            },
        )
}
