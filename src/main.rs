use legion::prelude::*;
use rltk::{GameState, Point, Rltk};

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
mod random_table;
mod saveload_system;
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
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
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
        let mut runstate = *self.resources.get::<RunState>().unwrap();

        ctx.cls();

        match runstate {
            RunState::MainMenu { .. } => {}
            _ => {
                draw_map(self, ctx);

                let map = self.resources.get::<Map>().unwrap();
                let query = <(Read<Position>, Read<Renderable>)>::query();
                let mut data = query.iter(&self.world).collect::<Vec<_>>();
                data.sort_by(|a, b| b.1.render_order.cmp(&a.1.render_order));
                for (pos, render) in data.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    if map.visible_tiles[idx] {
                        ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                    }
                }

                gui::draw_ui(&self.world, &self.resources, ctx);
            }
        }

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

            RunState::ShowInventory => {
                let (result, item) = gui::show_inventory(self, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => {
                        runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item = item.unwrap();

                        let mut ranged: Option<Ranged> = None;
                        if let Some(ranged_component) = self.world.get_component::<Ranged>(item) {
                            ranged = Some(*ranged_component);
                        }

                        if let Some(ranged) = ranged {
                            runstate = RunState::ShowTargeting {
                                range: ranged.range,
                                item,
                            }
                        } else {
                            self.world
                                .add_component(
                                    *self.resources.get::<Entity>().unwrap(),
                                    WantsToUseItem { item, target: None },
                                )
                                .expect("Unable to insert intent");
                            runstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let (result, item) = gui::drop_item_menu(self, ctx);
                match result {
                    gui::ItemMenuResult::Cancel => {
                        runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item = item.unwrap();
                        self.world
                            .add_component(
                                *self.resources.get::<Entity>().unwrap(),
                                WantsToDropItem { item },
                            )
                            .expect("Unable to insert intent");
                        runstate = RunState::PlayerTurn;
                    }
                }
            }

            RunState::ShowTargeting { range, item } => {
                let (result, target) = gui::ranged_target(self, ctx, range);
                match result {
                    gui::ItemMenuResult::Cancel => {
                        runstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        self.world
                            .add_component(
                                *self.resources.get::<Entity>().unwrap(),
                                WantsToUseItem { item, target },
                            )
                            .expect("Unable to insert intent");
                        runstate = RunState::PlayerTurn;
                    }
                }
            }

            RunState::MainMenu { .. } => {
                let result = gui::main_menu(self, ctx);
                match result {
                    gui::MainMenuResult::NoSelection { selected } => {
                        runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui::MainMenuResult::Selected { selected } => match selected {
                        gui::MainMenuSelection::NewGame => runstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.world);
                            runstate = RunState::PreRun;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.world, &*self.resources.get::<Map>().unwrap());
                runstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::LoadGame,
                };
            }

            RunState::NextLevel => {
                self.goto_next_level();
                runstate = RunState::PreRun;
            }
        }

        self.resources.insert(runstate);

        damage_system::delete_the_dead(&mut self.world, &mut self.resources);
    }
}

impl State {
    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        self.world
            .iter_entities()
            .filter(|entity| {
                // Don't delete the player
                if let Some(_player) = self.world.get_tag::<Player>(*entity) {
                    return false;
                }

                // Don't delete the player's equipment
                if self.world.has_component::<InBackpack>(*entity) {
                    return false;
                }
                if self.world.has_component::<Equipped>(*entity) {
                    return false;
                }

                // To Hades with it!
                true
            })
            .collect()
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        for target in self.entities_to_remove_on_level_change() {
            self.world.delete(target);
        }

        // Build a new map and place the player
        let current_map = self.resources.remove::<Map>().unwrap();
        let map = Map::new_map_rooms_and_corridors(current_map.depth + 1);

        // Spawn bad guys
        for room in map.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.world, &mut self.resources, room, map.depth);
        }

        // Place the player and update resources
        let (player_x, player_y) = map.rooms[0].center();
        self.resources.insert(Point::new(player_x, player_y));
        self.resources.insert(map);

        let player_entity = self.resources.get::<Entity>().unwrap();
        if let Some(mut player_pos) = self.world.get_component_mut::<Position>(*player_entity) {
            player_pos.x = player_x;
            player_pos.y = player_y;
        }

        // Mark the player's visibility dirty
        if let Some(mut viewshed) = self.world.get_component_mut::<Viewshed>(*player_entity) {
            viewshed.dirty = true;
        }

        // Notify the player and give one some health
        let mut gamelog = self.resources.get_mut::<gamelog::GameLog>().unwrap();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        if let Some(mut health) = self.world.get_component_mut::<CombatStats>(*player_entity) {
            health.hp = i32::max(health.hp, health.max_hp / 2);
        }
    }
}

fn main() -> rltk::BError {
    use rltk::{RandomNumberGenerator, RltkBuilder};
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()
        .unwrap();
    context.with_post_scanlines(true);

    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = Resources::default();

    let rng = RandomNumberGenerator::new();
    resources.insert(rng);
    resources.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    resources.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    let map = Map::new_map_rooms_and_corridors(1);

    let (player_x, player_y) = map.rooms[0].center();
    resources.insert(Point::new(player_x, player_y));

    let player = spawner::player(&mut world, player_x, player_y);
    resources.insert(player);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut world, &mut resources, room, map.depth);
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
            .add_system(inventory_system::item_use()) // Process WantsToUseItem
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
    rltk::main_loop(context, gs)
}
