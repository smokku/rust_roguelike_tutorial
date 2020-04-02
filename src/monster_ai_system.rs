use super::{Monster, Viewshed};
use legion::prelude::*;
use rltk::{console, Point};

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("monster_ai")
        .read_resource::<Point>()
        .with_query(Read::<Viewshed>::query().filter(tag::<Monster>()))
        .build(|_, world, player_pos, query| {
            for viewshed in query.iter(&world) {
                if viewshed.visible_tiles.contains(&**player_pos) {
                    console::log(format!("Monster shouts insults"));
                }
            }
        })
}
