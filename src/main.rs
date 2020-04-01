use legion::prelude::*;
use rltk::{Console, GameState, Rltk, VirtualKeyCode, RGB};
use std::cmp::{max, min};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LeftMover;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Player;

struct State {
    universe: Universe,
    world: World,
    schedule: Schedule,
}

fn try_move_player(delta_x: i32, delta_y: i32, world: &mut World) {
    let query = Write::<Position>::query().filter(tag::<Player>());

    for mut pos in query.iter(world) {
        pos.x = min(79, max(0, pos.x + delta_x));
        pos.y = min(49, max(0, pos.y + delta_y));
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.world),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.world),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.world),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.world),
            _ => {}
        },
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.schedule.execute(&mut self.world);

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
