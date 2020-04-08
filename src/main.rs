use legion::prelude::*;
use rltk::{Console, GameState, Point, Rltk};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod damage_system;
mod gamelog;
mod gui;
mod inventory_system;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod spawner;
mod visibility_system;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    WorldTurn,
    ShowInventory,
    ShowDropItem,
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
        {
            ctx.cls();
            draw_map(self, ctx);

            let map = self.resources.get::<Map>().unwrap();
            let query = <(Read<Position>, Read<Renderable>)>::query();
            for (pos, render) in query.iter(&self.world) {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }

            gui::draw_ui(&self.world, &self.resources, ctx);
        }

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
            RunState::ShowInventory => match gui::show_inventory(self, ctx) {
                gui::ItemMenuResult::Cancel => {
                    runstate = RunState::AwaitingInput;
                }
                gui::ItemMenuResult::NoResponse => {}
                gui::ItemMenuResult::Selected(item_entity) => {
                    // FIXME: This is hard-coded, as only Items we have so far are Potions
                    self.world
                        .add_component(
                            *self.resources.get::<Entity>().unwrap(),
                            WantsToDrinkPotion {
                                potion: item_entity,
                            },
                        )
                        .expect("Unable to insert intent");
                    runstate = RunState::PlayerTurn
                }
            },
            RunState::ShowDropItem => match gui::drop_item_menu(self, ctx) {
                gui::ItemMenuResult::Cancel => {
                    runstate = RunState::AwaitingInput;
                }
                gui::ItemMenuResult::NoResponse => {}
                gui::ItemMenuResult::Selected(item_entity) => {
                    // FIXME: This is hard-coded, as only Items we have so far are Potions
                    self.world
                        .add_component(
                            *self.resources.get::<Entity>().unwrap(),
                            WantsToDropItem { item: item_entity },
                        )
                        .expect("Unable to insert intent");
                    runstate = RunState::PlayerTurn
                }
            },
        }

        self.resources.insert(runstate);

        damage_system::delete_the_dead(&mut self.world, &mut self.resources);
    }
}

fn main() {
    use rltk::{RandomNumberGenerator, RltkBuilder};
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();
    context.with_post_scanlines(true);

    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = Resources::default();

    let rng = RandomNumberGenerator::new();
    resources.insert(rng);
    resources.insert(RunState::PreRun);
    resources.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    let map = Map::new_map_rooms_and_corridors();

    let (player_x, player_y) = map.rooms[0].center();
    resources.insert(Point::new(player_x, player_y));

    let player = spawner::player(&mut world, player_x, player_y);
    resources.insert(player);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut world, &mut resources, room);
    }

    resources.insert(map);

    let schedules = vec![
        Schedule::builder()
            .add_system(visibility_system::build())
            .build(),
        Schedule::builder()
            .add_system(monster_ai_system::build())
            .add_system(melee_combat_system::build()) // Creates SufferDamage out of WantsToMelee
            .add_system(damage_system::build()) // Turns SufferDamage to HP reduction
            .add_system(inventory_system::build()) // Turns WantsToPickupItem into InBackpack
            .add_system(inventory_system::item_drop()) // Turns WantsToDropItem into Position
            .add_system(inventory_system::potion_use()) // Turns WantsToDrinkPotion into HP changes
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
