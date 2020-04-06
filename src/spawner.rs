use super::{BlocksTile, CombatStats, Monster, Name, Player, Position, Renderable, Viewshed};
use legion::prelude::*;
use rltk::{RandomNumberGenerator, RGB};

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
