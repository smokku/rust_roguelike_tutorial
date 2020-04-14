use super::{components::*, gamelog::GameLog, Map, RunState, State};
use legion::prelude::*;
use rltk::{FontCharType, Point, Rltk, VirtualKeyCode, RGB};

pub fn draw_ui(world: &World, resources: &Resources, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let query = Read::<CombatStats>::query().filter(tag::<Player>());
    for stats in query.iter(&world) {
        let health = format!(" HP: {} / {}", stats.hp, stats.max_hp);
        ctx.print_color(
            12,
            43,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            &health,
        );
        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
        );
    }

    let map = resources.get::<Map>().unwrap();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(
        2,
        43,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        depth,
    );

    let log = resources.get::<GameLog>().unwrap();
    let mut y = 44;
    for s in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
    draw_tooltips(world, resources, ctx);
}

fn draw_tooltips(world: &World, resources: &Resources, ctx: &mut Rltk) {
    let map = resources.get::<Map>().unwrap();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip = Vec::new();
    let query = <(Read<Name>, Read<Position>)>::query();
    for (name, position) in query.iter(&world) {
        // FIXME: Should check against Player viewshed
        // as it is possible to reveal entities by hovering over map
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 {
                width = s.len() as i32;
            }
        }
        width += 3;

        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"->".to_string(),
            );
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(
                    left_x + 1,
                    y,
                    RGB::named(rltk::WHITE),
                    RGB::named(rltk::GREY),
                    s,
                );
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(
                        arrow_pos.x + width + 1 - i,
                        y,
                        RGB::named(rltk::WHITE),
                        RGB::named(rltk::GREY),
                        &" ".to_string(),
                    );
                }
                y += 1;
            }
            ctx.print_color(
                arrow_pos.x,
                arrow_pos.y,
                RGB::named(rltk::WHITE),
                RGB::named(rltk::GREY),
                &"<-".to_string(),
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.resources.get::<Entity>().unwrap();

    let query = <(Read<InBackpack>, Read<Name>)>::query();
    let items: Vec<(Entity, String)> = query
        .iter_entities(&gs.world)
        .filter(|(_entity, (pack, _name))| pack.owner == *player_entity)
        .map(|(entity, (_pack, name))| (entity, name.name.clone()))
        .collect();

    let count = items.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to cancel",
    );

    for (j, (_entity, name)) in items.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('a') + j as FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection >= 0 && selection < count as i32 {
                    (ItemMenuResult::Selected, Some(items[selection as usize].0))
                } else {
                    (ItemMenuResult::NoResponse, None)
                }
            }
        },
    }
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.resources.get::<Entity>().unwrap();

    let query = <(Read<InBackpack>, Read<Name>)>::query();
    let items: Vec<(Entity, String)> = query
        .iter_entities(&gs.world)
        .filter(|(_entity, (pack, _name))| pack.owner == *player_entity)
        .map(|(entity, (_pack, name))| (entity, name.name.clone()))
        .collect();

    let count = items.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Drop Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to cancel",
    );

    for (j, (_entity, name)) in items.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('a') + j as FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection >= 0 && selection < count as i32 {
                    (ItemMenuResult::Selected, Some(items[selection as usize].0))
                } else {
                    (ItemMenuResult::NoResponse, None)
                }
            }
        },
    }
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.resources.get::<Entity>().unwrap();

    let query = <(Read<Equipped>, Read<Name>)>::query();
    let items: Vec<(Entity, String)> = query
        .iter_entities(&gs.world)
        .filter(|(_entity, (item, _name))| item.owner == *player_entity)
        .map(|(entity, (_item, name))| (entity, name.name.clone()))
        .collect();

    let count = items.len();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Remove Which Item?",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESC to cancel",
    );

    for (j, (_entity, name)) in items.iter().enumerate() {
        ctx.set(
            17,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('('),
        );
        ctx.set(
            18,
            y,
            RGB::named(rltk::YELLOW),
            RGB::named(rltk::BLACK),
            rltk::to_cp437('a') + j as FontCharType,
        );
        ctx.set(
            19,
            y,
            RGB::named(rltk::WHITE),
            RGB::named(rltk::BLACK),
            rltk::to_cp437(')'),
        );

        ctx.print(21, y, &name);
        y += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection >= 0 && selection < count as i32 {
                    (ItemMenuResult::Selected, Some(items[selection as usize].0))
                } else {
                    (ItemMenuResult::NoResponse, None)
                }
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.resources.get::<Entity>().unwrap();
    let player_pos = gs.resources.get::<Point>().unwrap();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    let visible = gs.world.get_component::<Viewshed>(*player_entity);
    if let Some(visible) = visible {
        // We have a viewshed
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, RGB::named(rltk::BLUE));
                available_cells.push(*idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_pos.0, mouse_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.resources.get::<RunState>().unwrap();

    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rust Roguelike Tutorial",
    );

    if let RunState::MainMenu {
        menu_selection: selected,
    } = *runstate
    {
        ctx.print_color_centered(
            24,
            if selected == MainMenuSelection::NewGame {
                RGB::named(rltk::MAGENTA)
            } else {
                RGB::named(rltk::WHITE)
            },
            RGB::named(rltk::BLACK),
            "Begin New Game",
        );
        if save_exists {
            ctx.print_color_centered(
                25,
                if selected == MainMenuSelection::LoadGame {
                    RGB::named(rltk::MAGENTA)
                } else {
                    RGB::named(rltk::WHITE)
                },
                RGB::named(rltk::BLACK),
                "Load Game",
            );
        }
        ctx.print_color_centered(
            26,
            if selected == MainMenuSelection::Quit {
                RGB::named(rltk::MAGENTA)
            } else {
                RGB::named(rltk::WHITE)
            },
            RGB::named(rltk::BLACK),
            "Quit",
        );

        return match ctx.key {
            None => MainMenuResult::NoSelection { selected },
            Some(key) => match key {
                VirtualKeyCode::Escape => MainMenuResult::NoSelection {
                    selected: MainMenuSelection::Quit,
                },
                VirtualKeyCode::Up => match selected {
                    MainMenuSelection::NewGame => MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    },
                    MainMenuSelection::LoadGame => MainMenuResult::NoSelection {
                        selected: MainMenuSelection::NewGame,
                    },
                    MainMenuSelection::Quit => MainMenuResult::NoSelection {
                        selected: if save_exists {
                            MainMenuSelection::LoadGame
                        } else {
                            MainMenuSelection::NewGame
                        },
                    },
                },
                VirtualKeyCode::Down => match selected {
                    MainMenuSelection::NewGame => MainMenuResult::NoSelection {
                        selected: if save_exists {
                            MainMenuSelection::LoadGame
                        } else {
                            MainMenuSelection::Quit
                        },
                    },
                    MainMenuSelection::LoadGame => MainMenuResult::NoSelection {
                        selected: MainMenuSelection::Quit,
                    },
                    MainMenuSelection::Quit => MainMenuResult::NoSelection {
                        selected: MainMenuSelection::NewGame,
                    },
                },
                VirtualKeyCode::Return => MainMenuResult::Selected { selected },
                _ => MainMenuResult::NoSelection { selected },
            },
        };
    }

    MainMenuResult::NoSelection {
        selected: MainMenuSelection::NewGame,
    }
}
