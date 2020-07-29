use super::{camera, components::*, gamelog::GameLog, rex_assets::RexAssets, Map, RunState, State};
use legion::prelude::*;
use rltk::{FontCharType, Point, Rltk, VirtualKeyCode, RGB};

pub fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_ui(world: &World, resources: &Resources, ctx: &mut Rltk) {
    use rltk::to_cp437;
    let box_gray = RGB::from_hex("#999999").expect("Cannot convert color");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);

    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray, black); // Overall box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray, black); // Map box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray, black); // Log box
    draw_hollow_box(ctx, 49, 0, 30, 8, box_gray, black); // Top-right panel
    ctx.set(0, 45, box_gray, black, to_cp437('├'));
    ctx.set(49, 8, box_gray, black, to_cp437('├'));
    ctx.set(49, 0, box_gray, black, to_cp437('┬'));
    ctx.set(49, 45, box_gray, black, to_cp437('┴'));
    ctx.set(79, 8, box_gray, black, to_cp437('┤'));
    ctx.set(79, 45, box_gray, black, to_cp437('┤'));

    // Draw the town name
    let map = resources.get::<Map>().unwrap();
    let mut name = map.name.clone();
    std::mem::drop(map);
    const MAX_TOWN_NAME_LENGTH: i32 = 44;
    name.truncate(MAX_TOWN_NAME_LENGTH as usize);
    let name_length = name.len() as i32;
    let x_pos = (MAX_TOWN_NAME_LENGTH - name_length) / 2;
    ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
    ctx.print_color(x_pos + 1, 0, white, black, name);
    ctx.set(x_pos + 1 + name_length, 0, box_gray, black, to_cp437('├'));

    // Draw stats
    let player = resources.get::<Entity>().unwrap();
    let stats = world.get_component::<Pools>(*player).unwrap();
    let health = format!(
        "Health: {}/{}",
        stats.hit_points.current, stats.hit_points.max
    );
    let mana = format!("Mana:   {}/{}", stats.mana.current, stats.mana.max);
    ctx.print_color(50, 1, white, black, &health);
    ctx.print_color(50, 2, white, black, &mana);
    ctx.draw_bar_horizontal(
        64,
        1,
        14,
        stats.hit_points.current,
        stats.hit_points.max,
        RGB::named(rltk::RED),
        RGB::named(rltk::BLACK),
    );
    ctx.draw_bar_horizontal(
        64,
        2,
        14,
        stats.hit_points.current,
        stats.hit_points.max,
        RGB::named(rltk::BLUE),
        RGB::named(rltk::BLACK),
    );

    // Attributes
    let attr = world.get_component::<Attributes>(*player).unwrap();
    draw_attribute("Might:", &attr.might, 4, ctx);
    draw_attribute("Quickness:", &attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &attr.intelligence, 7, ctx);

    // Equipped
    let mut y = 9;
    let query = <(Read<Equipped>, Read<Name>)>::query();
    for (equipped_by, item_name) in query.iter(world) {
        if equipped_by.owner == *player {
            ctx.print_color(50, y, white, black, &item_name.name);
            y += 1;
        }
    }

    // Consumables
    y += 1;
    let green = RGB::from_f32(0.0, 1.0, 0.0);
    let yellow = RGB::named(rltk::YELLOW);
    let query = <(Read<InBackpack>, Read<Name>)>::query().filter(tag::<Consumable>());
    let mut index = 1;
    for (carried_by, item_name) in query.iter(world) {
        if carried_by.owner == *player && index < 10 {
            ctx.print_color(50, y, yellow, black, &format!("↑{}", index));
            ctx.print_color(53, y, green, black, &item_name.name);
            y += 1;
            index += 1;
        }
    }

    // Status
    let hunger = world.get_component::<HungerClock>(*player).unwrap();
    match hunger.state {
        HungerState::WellFed => ctx.print_color(
            50,
            44,
            RGB::named(rltk::GREEN),
            RGB::named(rltk::BLACK),
            "Well Fed",
        ),
        HungerState::Normal => {}
        HungerState::Hungry => ctx.print_color(
            50,
            44,
            RGB::named(rltk::ORANGE),
            RGB::named(rltk::BLACK),
            "Hungry",
        ),
        HungerState::Starving => ctx.print_color(
            50,
            44,
            RGB::named(rltk::RED),
            RGB::named(rltk::BLACK),
            "Starving",
        ),
    }

    // Draw the log
    let log = resources.get::<GameLog>().unwrap();
    let mut y = 46;
    for s in log.entries.iter().rev() {
        if y < 59 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    draw_tooltips(world, resources, ctx);
}

fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Oops");
    ctx.print_color(50, y, attr_gray, black, name);
    let color: RGB = if attribute.modifiers < 0 {
        RGB::from_f32(1.0, 0.0, 0.0)
    } else if attribute.modifiers == 0 {
        RGB::named(rltk::WHITE)
    } else {
        RGB::from_f32(0.0, 1.0, 0.0)
    };
    ctx.print_color(
        67,
        y,
        color,
        black,
        &format!("{}", attribute.base + attribute.modifiers),
    );
    ctx.print_color(73, y, color, black, &format!("{}", attribute.bonus));
    if attribute.bonus > 0 {
        ctx.set(72, y, color, black, rltk::to_cp437('+'));
    }
}

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Self {
        Tooltip { lines: Vec::new() }
    }

    fn add<S: ToString>(&mut self, line: S) {
        self.lines.push(line.to_string());
    }

    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.len() > max {
                max = s.len();
            }
        }
        max as i32 + 2i32
    }

    fn height(&self) -> i32 {
        self.lines.len() as i32 + 2i32
    }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, white, box_gray);
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, &s);
        }
    }
}

fn draw_tooltips(world: &World, resources: &Resources, ctx: &mut Rltk) {
    use rltk::to_cp437;

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(resources, ctx);
    let map = resources.get::<Map>().unwrap();

    let mouse_pos = ctx.mouse_pos();
    let mouse_map_pos = (mouse_pos.0 + min_x - 1, mouse_pos.1 + min_y - 1);
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] {
        return;
    }

    let mut tip_boxes = Vec::new();

    let query = <(Read<Name>, Read<Position>)>::query().filter(!tag::<Hidden>());
    for (entity, (name, position)) in query.iter_entities(world) {
        if position.x == mouse_map_pos.0 && position.y == mouse_map_pos.1 {
            let mut tip = Tooltip::new();

            tip.add(name.name.to_string());

            // Comment on attributes
            if let Some(attr) = world.get_component::<Attributes>(entity) {
                let mut s = "".to_string();
                if attr.might.bonus < 0 {
                    s += "Weak. "
                };
                if attr.might.bonus > 0 {
                    s += "Strong. "
                };
                if attr.quickness.bonus < 0 {
                    s += "Clumsy. "
                };
                if attr.quickness.bonus > 0 {
                    s += "Agile. "
                };
                if attr.fitness.bonus < 0 {
                    s += "Unhealthy. "
                };
                if attr.fitness.bonus > 0 {
                    s += "Healthy. "
                };
                if attr.intelligence.bonus < 0 {
                    s += "Unintelligent. "
                };
                if attr.intelligence.bonus > 0 {
                    s += "Smart. "
                };
                if s.is_empty() {
                    s = "Quite Average.".to_string();
                }
                tip.add(s);
            }

            // Comment on pools
            if let Some(stat) = world.get_component::<Pools>(entity) {
                tip.add(format!("Level: {}", stat.level));
            }

            tip_boxes.push(tip);
        }
    }

    if tip_boxes.is_empty() {
        return;
    }

    let box_gray = RGB::from_hex("#999999").expect("Oops");
    let white = RGB::named(rltk::WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    let tooltip_left = mouse_pos.0 > 40;
    if tooltip_left {
        // Render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    ctx.set(arrow_x, arrow_y, white, box_gray, arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.height();
    }

    let mut y = mouse_pos.1 - total_height / 2;
    if y + total_height / 2 > 50 {
        y -= total_height / 2 - 50;
    }

    for tt in tip_boxes.iter() {
        let x = if tooltip_left {
            mouse_pos.0 - (1 + tt.width())
        } else {
            mouse_pos.0 + (1 + 1)
        };
        tt.render(ctx, x, y);
        y += tt.height();
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
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.resources, ctx);
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
                let screen_x = idx.x - min_x;
                let screen_y = idx.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                    available_cells.push(*idx);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mouse_map_pos = (mouse_pos.0 + min_x - 1, mouse_pos.1 + min_y - 1);
    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 {
            valid_target = true;
        }
    }
    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
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
    let assets = gs.resources.get::<RexAssets>().unwrap();

    ctx.render_xp_sprite(&assets.menu, 0, 0);

    ctx.draw_box_double(
        24,
        18,
        31,
        10,
        RGB::named(rltk::WHEAT),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color_centered(
        20,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rust Roguelike Tutorial",
    );
    ctx.print_color_centered(
        21,
        RGB::named(rltk::CYAN),
        RGB::named(rltk::BLACK),
        "by Herbert Wolverson",
    );
    ctx.print_color_centered(
        22,
        RGB::named(rltk::GRAY),
        RGB::named(rltk::BLACK),
        "Use Up/Down Arrows and Enter",
    );

    let mut y = 24;
    if let RunState::MainMenu {
        menu_selection: selected,
    } = *runstate
    {
        ctx.print_color_centered(
            y,
            if selected == MainMenuSelection::NewGame {
                RGB::named(rltk::MAGENTA)
            } else {
                RGB::named(rltk::WHITE)
            },
            RGB::named(rltk::BLACK),
            "Begin New Game",
        );
        y += 1;

        if save_exists {
            ctx.print_color_centered(
                y,
                if selected == MainMenuSelection::LoadGame {
                    RGB::named(rltk::MAGENTA)
                } else {
                    RGB::named(rltk::WHITE)
                },
                RGB::named(rltk::BLACK),
                "Load Game",
            );
            y += 1;
        }

        ctx.print_color_centered(
            y,
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

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Your journey has ended!",
    );
    ctx.print_color_centered(
        17,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "One day, we'll tell you all about how you did.",
    );
    ctx.print_color_centered(
        18,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "That day, sadly, is not in this chapter..",
    );

    ctx.print_color_centered(
        20,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press any key to return to the menu.",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
