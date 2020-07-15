use super::{
    a_star_search, particle_system::ParticleBuilder, Confusion, Map, Monster, Position, RunState,
    Viewshed, WantsToMelee,
};
use legion::prelude::*;
use rltk::Point;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("monster_ai")
        .write_resource::<Map>()
        .read_resource::<Point>()
        .read_resource::<Entity>()
        .read_resource::<RunState>()
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Monster>()))
        .write_component::<Confusion>()
        .write_resource::<ParticleBuilder>()
        .build(
            |command_buffer,
             world,
             (map, player_pos, player_entity, runstate, particle_builder),
             query| unsafe {
                if **runstate != RunState::MonsterTurn {
                    return;
                }
                for (entity, (mut viewshed, mut pos)) in query.iter_entities_unchecked(world) {
                    let mut can_act = true;

                    if let Some(mut confused) =
                        world.get_component_mut_unchecked::<Confusion>(entity)
                    {
                        confused.turns -= 1;
                        if confused.turns < 1 {
                            command_buffer.remove_component::<Confusion>(entity);
                        }
                        can_act = false;
                        particle_builder.request(
                            pos.x,
                            pos.y,
                            rltk::RGB::named(rltk::MAGENTA),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('?'),
                            200.0,
                        );
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
                            if let Some((path, _cost)) =
                                a_star_search(Point::new(pos.x, pos.y), **player_pos, &**map)
                            {
                                if path.len() > 1 {
                                    let mut idx = map.xy_idx(pos.x, pos.y);
                                    map.blocked[idx] = false;
                                    pos.x = path[1].x;
                                    pos.y = path[1].y;
                                    idx = map.xy_idx(pos.x, pos.y);
                                    map.blocked[idx] = true;
                                    viewshed.dirty = true;
                                }
                            }
                        }
                    }
                }
            },
        )
}
