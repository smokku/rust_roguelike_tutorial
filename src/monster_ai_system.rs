use super::{Map, Monster, Name, Position, Viewshed};
use legion::prelude::*;
use rltk::{console, Point};

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("monster_ai")
        .write_resource::<Map>()
        .read_resource::<Point>()
        .with_query(
            <(Write<Viewshed>, Read<Name>, Write<Position>)>::query().filter(tag::<Monster>()),
        )
        .build(|_, mut world, (map, player_pos), query| {
            for (mut viewshed, name, mut pos) in query.iter_mut(&mut world) {
                let distance = rltk::DistanceAlg::Pythagoras
                    .distance2d(Point::new(pos.x, pos.y), **player_pos);
                if distance < 1.5 {
                    // Attack goes here
                    console::log(&format!("{} shouts insults", name.name));
                    return;
                }
                if viewshed.visible_tiles.contains(&**player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &**map,
                    );
                    if path.success && path.steps.len() > 1 {
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        viewshed.dirty = true;
                    }
                }
            }
        })
}
