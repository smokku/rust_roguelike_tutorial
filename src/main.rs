use legion::prelude::*;
use rltk::{Console, GameState, Rltk, RGB};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod visibility_system;

pub struct State {
    pub universe: Universe,
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(&mut self.world, &mut self.resources, ctx);

        self.schedule.execute(&mut self.world, &mut self.resources);

        draw_map(&mut self.world, &mut self.resources, ctx);

        let query = <(Read<Position>, Read<Renderable>)>::query();
        for (pos, render) in query.iter(&mut self.world) {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();

    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = Resources::default();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    resources.insert(map);

    world.insert(
        (Player,),
        vec![(
            Position {
                x: player_x,
                y: player_y,
            },
            Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
            },
        )],
    );

    let schedule = Schedule::builder()
        .add_system(visibility_system::build())
        .build();

    let gs = State {
        universe,
        world,
        resources,
        schedule,
    };
    rltk::main_loop(context, gs);
}
