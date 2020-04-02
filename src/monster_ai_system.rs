use super::{Map, Monster, Position, Viewshed};
use legion::prelude::*;
use rltk::{console, field_of_view, Point};

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("monster_ai")
        .with_query(<(Read<Viewshed>, Read<Position>)>::query().filter(tag::<Monster>()))
        .build(|_, mut world, _, query| {
            for (viewshed, pos) in query.iter(world) {
                console::log("Monster considers their own existence");
            }
        })
}
