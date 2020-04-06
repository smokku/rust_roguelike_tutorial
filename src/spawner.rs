use super::{
    map::MAP_WIDTH, BlocksTile, CombatStats, Item, Monster, Name, Player, Position, Potion, Rect,
    Renderable, Viewshed,
};
use legion::prelude::*;
use rltk::{RandomNumberGenerator, RGB};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

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
    )[0]
}

// Spawns a random monster at a given location.
pub fn random_monster(world: &mut World, resources: &mut Resources, x: i32, y: i32) {
    let mut rng = resources.get_mut::<RandomNumberGenerator>().unwrap();
    let roll = rng.roll_dice(1, 2);
    match roll {
        1 => {
            orc(world, x, y);
        }
        _ => {
            goblin(world, x, y);
        }
    }
}

fn orc(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, rltk::to_cp437('o'), "Orc");
}
fn goblin(world: &mut World, x: i32, y: i32) {
    monster(world, x, y, rltk::to_cp437('g'), "Goblin");
}

fn monster(world: &mut World, x: i32, y: i32, glyph: u8, name: &str) {
    world.insert(
        (Monster, BlocksTile),
        vec![(
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
        (Item,),
        vec![(
            Position { x, y },
            Renderable {
                glyph: rltk::to_cp437('¡'),
                fg: RGB::named(rltk::MAGENTA),
                bg: RGB::named(rltk::BLACK),
            },
            Name {
                name: "Health Potion".to_string(),
            },
            Potion { heal_amount: 8 },
        )],
    );
}

pub fn spawn_room(world: &mut World, resources: &mut Resources, room: &Rect) {
    let mut monster_spawn_points = Vec::new();
    let mut item_spawn_points = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = resources.get_mut::<RandomNumberGenerator>().unwrap();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        random_monster(world, resources, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % MAP_WIDTH;
        let y = *idx / MAP_WIDTH;
        health_potion(world, x as i32, y as i32);
    }
}
