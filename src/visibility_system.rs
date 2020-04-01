use super::{Map, Position, Viewshed};
use legion::prelude::*;
use rltk::{field_of_view, Point};

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("visibility_system")
        .read_resource::<Map>()
        .with_query(<(Write<Viewshed>, Read<Position>)>::query())
        .build(|_, mut world, map, query| {
            for (mut viewshed, pos) in query.iter_mut(&mut world) {
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &**map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1);
            }
        })
}
