use super::{Bystander, Map, Position, RunState, Viewshed};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("bystander_ai")
        .with_query(<(Write<Viewshed>, Write<Position>)>::query().filter(tag::<Bystander>()))
        .write_resource::<Map>()
        .read_resource::<RunState>()
        .write_resource::<rltk::RandomNumberGenerator>()
        .build(|_, world, (map, runstate, rng), query| {
            if **runstate != RunState::MonsterTurn {
                return;
            }

            for (mut viewshed, mut pos) in query.iter_mut(world) {
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
        })
}
