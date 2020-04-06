use legion::prelude::*;
use rltk::{Console, GameState, Point, Rltk, RGB};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod damage_system;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod visibility_system;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    WorldTurn,
}

pub struct State {
    pub universe: Universe,
    pub world: World,
    pub resources: Resources,
    pub schedules: Vec<Schedule>,
}

impl State {
    fn run_systems(&mut self) {
        for schedule in self.schedules.iter_mut() {
            schedule.execute(&mut self.world, &mut self.resources);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut runstate = *self.resources.get::<RunState>().unwrap();

        match runstate {
            RunState::PreRun => {
                self.run_systems();
                runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                runstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                runstate = RunState::WorldTurn;
            }
            RunState::WorldTurn => {
                self.run_systems();
                runstate = RunState::AwaitingInput;
            }
        }

        self.resources.insert(runstate);

        damage_system::delete_the_dead(&mut self.world, &mut self.resources);

        draw_map(self, ctx);

        let map = self.resources.get::<Map>().unwrap();
        let query = <(Read<Position>, Read<Renderable>)>::query();
        for (pos, render) in query.iter(&self.world) {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

fn main() {
    use rltk::{RandomNumberGenerator, RltkBuilder};
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();

    let mut rng = RandomNumberGenerator::new();

    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = Resources::default();

    resources.insert(RunState::PreRun);

    let map = Map::new_map_rooms_and_corridors();

    let (player_x, player_y) = map.rooms[0].center();
    resources.insert(Point::new(player_x, player_y));

    let player = world.insert(
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
                dirty: true,
            },
            Name {
                name: "Player".to_string(),
            },
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
        )],
    );
    resources.insert(player[0]);

    world.insert(
        (Monster, BlocksTile),
        map.rooms.iter().skip(1).enumerate().map(|(i, room)| {
            let (x, y) = room.center();

            let glyph: u8;
            let name: String;
            let roll = rng.roll_dice(1, 2);
            match roll {
                1 => {
                    glyph = rltk::to_cp437('g');
                    name = "Goblin".to_string();
                }
                _ => {
                    glyph = rltk::to_cp437('o');
                    name = "Orc".to_string();
                }
            }
            (
                Position { x, y },
                Renderable {
                    glyph,
                    fg: RGB::named(rltk::RED),
                    bg: RGB::named(rltk::BLACK),
                },
                Viewshed {
                    visible_tiles: Vec::new(),
                    range: 8,
                    dirty: true,
                },
                Name {
                    name: format!("{} #{}", name, i),
                },
                CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                },
            )
        }),
    );

    resources.insert(map);

    let schedules = vec![
        Schedule::builder()
            .add_system(visibility_system::build())
            .build(),
        Schedule::builder()
            .add_system(monster_ai_system::build())
            .add_system(melee_combat_system::build()) // Creates SufferDamage out of WantsToMelee
            .add_system(damage_system::build()) // Turns SufferDamage to HP reduction
            .build(),
        Schedule::builder()
            .add_system(map_indexing_system::build())
            .build(),
    ];

    let gs = State {
        universe,
        world,
        resources,
        schedules,
    };
    rltk::main_loop(context, gs);
}
