use super::{components::*, prefabs::*, random_table::RandomTable, Map, Rect, TileType};
use legion::prelude::*;
use rltk::{FontCharType, RandomNumberGenerator, RGB};
use std::collections::HashMap;

const MAX_MONSTERS: i32 = 4;

// Spawns the player and returns the entity object.
pub fn player(world: &mut World, x: i32, y: i32) -> Entity {
    world.insert(
        (Player,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('@'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
                render_order: 0,
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
            HungerClock {
                state: HungerState::WellFed,
                duration: 20,
            },
        )],
    )[0]
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
        .add("Longsword", map_depth - 1)
        .add("TowerShield", map_depth - 1)
        .add("Rations", 10)
        .add("Magic Mapping Scroll", 2)
        .add("Bear Trap", 2)
}

pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets = Vec::new();

    // Borrow scope - to keep access to the map isolated
    {
        for y in room.y1 + 1..room.y2 {
            for x in room.x1 + 1..room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points = HashMap::new();
    let mut areas = Vec::from(area);

    let num_spawns = i32::min(
        areas.len() as i32,
        rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
    );
    if num_spawns == 0 {
        return;
    }

    for _i in 0..num_spawns {
        let array_index = if area.len() == 1 {
            0
        } else {
            (rng.roll_dice(1, areas.len() as i32) - 1) as usize
        };
        let map_idx = areas[array_index];
        spawn_points.insert(map_idx, spawn_table.roll(rng));
        areas.remove(array_index);
    }

    // Actually spawn the monsters
    for (idx, spawn) in spawn_points.iter() {
        spawn_list.push((*idx, (*spawn).clone()));
    }
}

// Spawn a named entity at the location
pub fn spawn_entity(world: &mut World, map: &Map, idx: &usize, name: &str) {
    let x = *idx as i32 % map.width;
    let y = *idx as i32 / map.width;

    let item_result = spawn_named_item(
        &PREFABS.lock().unwrap(),
        world,
        name,
        SpawnType::AtPosition { x, y },
    );
    if item_result.is_some() {
        return;
    }

    match name.as_ref() {
        "Goblin" => goblin(world, x, y),
        "Orc" => orc(world, x, y),
        "Dagger" => dagger(world, x, y),
        "Shield" => shield(world, x, y),
        "Longsword" => longsword(world, x, y),
        "Tower Shield" => tower_shield(world, x, y),
        "Bear Trap" => bear_trap(world, x, y),
        "Door" => door(world, x, y),
        _ => {}
    }
}

fn orc(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, rltk::to_cp437('o'), "Orc");
}
fn goblin(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, rltk::to_cp437('g'), "Goblin");
}

fn monster(world: &mut World, x: i32, y: i32, glyph: FontCharType, name: &str) {
    world.insert(
        (Monster, BlocksTile),
        vec![(
            Position { x, y },
            Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                render_order: 1,
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            },
            Name {
                name: name.to_string(),
            },
            CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            },
        )],
    );
}

fn dagger(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('/'),
                fg: RGB::named(rltk::CYAN),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Dagger".to_string(),
            },
            Equippable {
                slot: EquipmentSlot::Melee,
            },
            MeleePowerBonus { power: 2 },
        )],
    );
}

fn shield(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('('),
                fg: RGB::named(rltk::CYAN),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Shield".to_string(),
            },
            Equippable {
                slot: EquipmentSlot::Shield,
            },
            DefenseBonus { defense: 1 },
        )],
    );
}

fn longsword(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('/'),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Longsword".to_string(),
            },
            Equippable {
                slot: EquipmentSlot::Melee,
            },
            MeleePowerBonus { power: 4 },
        )],
    );
}

fn tower_shield(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('('),
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Tower Shield".to_string(),
            },
            Equippable {
                slot: EquipmentSlot::Shield,
            },
            DefenseBonus { defense: 3 },
        )],
    );
}

fn bear_trap(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Hidden, EntryTrigger, SingleActivation),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('^'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Bear Trap".to_string(),
            },
            InflictsDamage { damage: 6 },
        )],
    );
}

fn door(world: &mut World, x: i32, y: i32) {
    world.insert(
        (BlocksTile, BlocksVisibility),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('+'),
                fg: RGB::named(rltk::CHOCOLATE),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Door".to_string(),
            },
            Door { open: false },
        )],
    );
}
