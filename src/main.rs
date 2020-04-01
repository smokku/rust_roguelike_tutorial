use legion::prelude::*;
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};

// #[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

// #[derive(Clone, Copy, Debug, PartialEq)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeftMover;

struct State {
    universe: Universe,
    world: World,
    schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let Self { world, .. } = self;

        self.schedule.execute(world);

        let query = <(Read<Position>, Read<Renderable>)>::query();
        for (pos, render) in query.iter(world) {
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

    world.insert(
        (),
        vec![(
            Position { x: 40, y: 25 },
            Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
            },
        )],
    );

    world.insert(
        (LeftMover,),
        (0..10).map(|i| {
            (
                Position { x: i * 7, y: 20 },
                Renderable {
                    glyph: rltk::to_cp437('☺'),
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                },
            )
        }),
    );

    let left_walker = SystemBuilder::new("left_walker")
        .with_query(Write::<Position>::query().filter(tag::<LeftMover>()))
        .build(|_, mut world, (), query| {
            for mut pos in query.iter(&mut world) {
                pos.x -= 1;
                if pos.x < 0 {
                    pos.x = 79;
                }
            }
        });

    let schedule = Schedule::builder().add_system(left_walker).build();

    let gs = State {
        universe,
        world,
        schedule,
    };
    rltk::main_loop(context, gs);
}
