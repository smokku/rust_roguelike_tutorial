use super::{components::*, map::MAP_WIDTH, random_table::RandomTable, Rect};
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

#[allow(clippy::map_entry)]
pub fn spawn_room(world: &mut World, resources: &mut Resources, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = resources.get_mut::<RandomNumberGenerator>().unwrap();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for (idx, spawn) in spawn_points.iter() {
        let x = (*idx % MAP_WIDTH) as i32;
        let y = (*idx / MAP_WIDTH) as i32;

        match spawn.as_ref() {
            "Goblin" => goblin(world, x, y),
            "Orc" => orc(world, x, y),
            "Health Potion" => health_potion(world, x, y),
            "Fireball Scroll" => fireball_scroll(world, x, y),
            "Confusion Scroll" => confusion_scroll(world, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(world, x, y),
            "Dagger" => dagger(world, x, y),
            "Shield" => shield(world, x, y),
            "Longsword" => longsword(world, x, y),
            "Tower Shield" => tower_shield(world, x, y),
            "Rations" => rations(world, x, y),
            "Magic Mapping Scroll" => magic_mapping_scroll(world, x, y),
            "Bear Trap" => bear_trap(world, x, y),
            _ => {}
        }
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

fn health_potion(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('¡'),
                fg: RGB::named(rltk::MAGENTA),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Health Potion".to_string(),
            },
            ProvidesHealing { heal_amount: 8 },
        )],
    );
}

fn magic_missile_scroll(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437(')'),
                fg: RGB::named(rltk::CYAN),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Magic Missile Scroll".to_string(),
            },
            Ranged { range: 6 },
            InflictsDamage { damage: 8 },
        )],
    );
}

fn fireball_scroll(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437(')'),
                fg: RGB::named(rltk::ORANGE),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Fireball Scroll".to_string(),
            },
            Ranged { range: 6 },
            InflictsDamage { damage: 20 },
            AreaOfEffect { radius: 3 },
        )],
    );
}

fn confusion_scroll(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437(')'),
                fg: RGB::named(rltk::PINK),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Confusion Scroll".to_string(),
            },
            Ranged { range: 6 },
            Confusion { turns: 4 },
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

fn rations(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, ProvidesFood, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('%'),
                fg: RGB::named(rltk::GREEN),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Rations".to_string(),
            },
        )],
    );
}

fn magic_mapping_scroll(world: &mut World, x: i32, y: i32) {
    world.insert(
        (Item, MagicMapper, Consumable),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437(')'),
                fg: RGB::named(rltk::CYAN3),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            },
            Name {
                name: "Scroll of Magic Mapping".to_string(),
            },
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
