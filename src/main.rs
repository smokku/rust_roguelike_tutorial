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

pub struct State {
    pub universe: Universe,
    pub world: World,
    pub resources: Resources,
    pub schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.schedule.execute(&mut self.world);

        let map = self.resources.get::<Vec<TileType>>().unwrap();
        draw_map(&map, ctx);

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

    resources.insert(new_map_rooms_and_corridors());

    world.insert(
        (Player,),
        vec![(
            Position { x: 40, y: 25 },
            Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        )],
    );

    let schedule = Schedule::builder().build();

    let gs = State {
        universe,
        world,
        resources,
        schedule,
    };
    rltk::main_loop(context, gs);
}
