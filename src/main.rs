use legion::prelude::*;
use rltk::{GameState, Point, RandomNumberGenerator, Rltk, RltkBuilder};

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::*;
mod camera;
mod damage_system;
mod gamelog;
mod gui;
mod hunger_system;
mod inventory_system;
mod map_builders;
mod map_indexing_system;
mod melee_combat_system;
mod monster_ai_system;
mod particle_system;
mod prefabs;
mod random_table;
mod rex_assets;
mod saveload_system;
mod spawner;
mod trigger_system;
mod visibility_system;

const SHOW_MAPGEN_VISUALIZER: bool = true;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    WorldTurn,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
}

pub struct State {
    pub universe: Universe,
    pub world: World,
    pub resources: Resources,
    pub schedules: Vec<Schedule>,

    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
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
        particle_system::cull_dead_particles(&mut self.world, ctx);

        match runstate {
            RunState::MainMenu { .. } | RunState::GameOver => {}
            _ => {
                camera::render_camera(&self.world, &self.resources, ctx);
                gui::draw_ui(&self.world, &self.resources, ctx);
            }
        }

        match runstate {
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    runstate = self.mapgen_next_state.unwrap();
                } else if self.mapgen_index >= self.mapgen_history.len() {
                    runstate = self.mapgen_next_state.unwrap();
                } else {
                    ctx.cls();
                    if self.mapgen_index < self.mapgen_history.len() {
                        camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                    }

                    const MAX_VISUALIZATION_TIME: i32 = 15000; // Let the visualization be around 15 seconds
                    let frame_timer: f32 = f32::max(
                        10.0, // not shorter than 10ms
                        f32::min(
                            300.0, // not longer than 300ms
                            MAX_VISUALIZATION_TIME as f32 / self.mapgen_history.len() as f32,
                        ),
                    );
                    self.mapgen_timer += ctx.frame_time_ms;
                    if self.mapgen_timer > frame_timer {
                        self.mapgen_timer -= frame_timer;
                        self.mapgen_index += 1;
                    }
                }
            }
            RunState::PreRun => {
                self.run_systems();
                runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                runstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                runstate = *self.resources.get::<RunState>().unwrap();
                if runstate == RunState::PlayerTurn {
                    runstate = RunState::MonsterTurn;
                }
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
            RunState::ShowRemoveItem => {
                let (result, item) = gui::remove_item_menu(self, ctx);
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
                                WantsToRemoveItem { item },
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

            RunState::GameOver => {
                let result = gui::game_over(ctx);
                match result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        runstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
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

            RunState::MagicMapReveal { row } => {
                let mut map = self.resources.get_mut::<Map>().unwrap();
                for x in 0..map.width {
                    let idx = map.xy_idx(x as i32, row);
                    map.revealed_tiles[idx] = true;
                }
                if row >= map.height - 1 {
                    runstate = RunState::MonsterTurn;
                } else {
                    runstate = RunState::MagicMapReveal { row: row + 1 };
                }
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

        // Build a new map and Place the player and update resources
        let current_map = self.resources.remove::<Map>().unwrap();
        self.generate_world_map(current_map.depth + 1);

        // Notify the player and give one some health
        let player_entity = self.resources.get::<Entity>().unwrap();
        let mut gamelog = self.resources.get_mut::<gamelog::GameLog>().unwrap();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        if let Some(mut health) = self.world.get_component_mut::<CombatStats>(*player_entity) {
            health.hp = i32::max(health.hp, health.max_hp / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        self.world.delete_all();

        // Clear gamelog
        {
            let mut log = self.resources.get_mut::<gamelog::GameLog>().unwrap();
            log.entries.clear();
        }

        // Spawn a new player
        self.resources
            .insert(spawner::player(&mut self.world, 0, 0));

        // Build a new map and place the player
        self.generate_world_map(1);
    }

    fn generate_world_map(&mut self, depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();

        // Build a new map
        let mut rng = self.resources.get_mut::<RandomNumberGenerator>().unwrap();
        let mut builder = map_builders::random_builder(depth, 80, 50, &mut rng);
        builder.build_map(&mut rng);
        std::mem::drop(rng); // do not borrow self anymore
        self.mapgen_history = builder.build_data.history.clone();
        let map = builder.build_data.map.clone();
        self.resources.insert(map);

        // Spawn bad guys
        builder.spawn_entities(&mut self.world);

        // Place the player and update resources
        let player_start = builder.build_data.starting_position.unwrap().clone();
        self.resources
            .insert(Point::new(player_start.x, player_start.y));

        let player_entity = self.resources.get::<Entity>().unwrap();
        if let Some(mut player_pos) = self.world.get_component_mut::<Position>(*player_entity) {
            player_pos.x = player_start.x;
            player_pos.y = player_start.y;
        }

        // Mark the player's visibility dirty
        if let Some(mut viewshed) = self.world.get_component_mut::<Viewshed>(*player_entity) {
            viewshed.dirty = true;
        }
    }
}

fn main() -> rltk::BError {
    let mut context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()
        .unwrap();
    context.with_post_scanlines(true);

    prefabs::load_prefabs();

    let universe = Universe::new();
    let mut world = universe.create_world();
    let mut resources = Resources::default();

    resources.insert(RandomNumberGenerator::new());
    resources.insert(particle_system::ParticleBuilder::new());
    resources.insert(rex_assets::RexAssets::new());

    resources.insert(RunState::MapGeneration {});
    resources.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rusty Roguelike".to_string()],
    });

    // Insert placeholder values for "Start Game" map generator
    resources.insert(Point::new(0, 0));
    resources.insert(spawner::player(&mut world, 0, 0));
    resources.insert(Map::new(1, 64, 64));

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
            .add_system(inventory_system::item_remove()) // Turns WantsToRemoveItem into InBackpack
            .add_system(inventory_system::item_use()) // Process WantsToUseItem
            .build(),
        Schedule::builder()
            .add_system(trigger_system::build())
            .add_system(map_indexing_system::build())
            .add_system(hunger_system::build()) // Process HungerClock
            .add_thread_local_fn(particle_system::particle_spawn()) // Turns ParticleRequests into particle Entities
            .build(),
    ];

    let mut gs = State {
        universe,
        world,
        resources,
        schedules,

        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    gs.generate_world_map(1);

    rltk::main_loop(context, gs)
}
