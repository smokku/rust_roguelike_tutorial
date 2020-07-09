use super::{gamelog::GameLog, Bystander, Map, Name, Point, Position, Quips, RunState, Viewshed};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("bystander_ai")
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Bystander>()))
        .write_resource::<Map>()
        .read_resource::<RunState>()
        .write_resource::<rltk::RandomNumberGenerator>()
        .read_resource::<Point>()
        .write_resource::<GameLog>()
        .write_component::<Quips>()
        .read_component::<Name>()
        .build(
            |_, world, (map, runstate, rng, player_pos, gamelog), query| unsafe {
                if **runstate != RunState::MonsterTurn {
                    return;
                }

                for (entity, (mut viewshed, mut pos)) in query.iter_entities_unchecked(world) {
                    // Possibly quip
                    if let Some(mut quips) = world.get_component_mut_unchecked::<Quips>(entity) {
                        if !quips.available.is_empty()
                            && viewshed.visible_tiles.contains(&player_pos)
                            && rng.roll_dice(1, 6) == 1
                        {
                            if let Some(name) = world.get_component::<Name>(entity) {
                                let quip = if quips.available.len() == 1 {
                                    0
                                } else {
                                    (rng.roll_dice(1, quips.available.len() as i32) - 1) as usize
                                };
                                gamelog.entries.push(format!(
                                    "{} says \"{}\"",
                                    name.name, quips.available[quip]
                                ));
                                quips.available.remove(quip);
                            }
                        }
                    }

                    // Try to move randomly
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    if x >= 0 && x < map.width - 1 && y >= 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !map.blocked[dest_idx] {
                            let idx = map.xy_idx(pos.x, pos.y);
                            map.blocked[idx] = false;
                            pos.x = x;
                            pos.y = y;
                            map.blocked[dest_idx] = true;
                            viewshed.dirty = true;
                        }
                    }
                }
            },
        )
}
