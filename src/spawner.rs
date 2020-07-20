use super::{components::*, prefabs::*, random_table::RandomTable, Map, Rect, TileType};
use crate::{attr_bonus, mana_at_level, player_hp_at_level};
use legion::prelude::*;
use rltk::{RandomNumberGenerator, RGB};
use std::collections::HashMap;

const MAX_MONSTERS: i32 = 4;
const BASE_ATTRIBUTE: i32 = 11;

// Spawns the player and returns the entity object.
pub fn player(world: &mut World, x: i32, y: i32) -> Entity {
    let mut skills = Skills {
        skills: HashMap::new(),
    };
    skills.skills.insert(Skill::Melee, 1);
    skills.skills.insert(Skill::Defense, 1);
    skills.skills.insert(Skill::Magic, 1);

    world.insert(
        (Player, BlocksTile),
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
            Attributes {
                might: Attribute {
                    base: 11,
                    modifiers: 0,
                    bonus: attr_bonus(BASE_ATTRIBUTE),
                },
                fitness: Attribute {
                    base: BASE_ATTRIBUTE,
                    modifiers: 0,
                    bonus: attr_bonus(BASE_ATTRIBUTE),
                },
                quickness: Attribute {
                    base: BASE_ATTRIBUTE,
                    modifiers: 0,
                    bonus: attr_bonus(BASE_ATTRIBUTE),
                },
                intelligence: Attribute {
                    base: BASE_ATTRIBUTE,
                    modifiers: 0,
                    bonus: attr_bonus(BASE_ATTRIBUTE),
                },
            },
            skills,
            Pools {
                hit_points: Pool {
                    current: player_hp_at_level(BASE_ATTRIBUTE, 1),
                    max: player_hp_at_level(BASE_ATTRIBUTE, 1),
                },
                mana: Pool {
                    current: mana_at_level(BASE_ATTRIBUTE, 1),
                    max: mana_at_level(BASE_ATTRIBUTE, 1),
                },
                experience: 0,
                level: 1,
            },
        )],
    )[0]
}

fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&PREFABS.lock().unwrap(), map_depth)
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
        if let Some(spawn) = spawn {
            spawn_list.push((*idx, (*spawn).clone()));
        }
    }
}

// Spawn a named entity at the location
pub fn spawn_entity(world: &mut World, map: &Map, idx: &usize, name: &str) {
    let x = *idx as i32 % map.width;
    let y = *idx as i32 / map.width;

    let item_result = spawn_named_entity(
        &PREFABS.lock().unwrap(),
        world,
        name,
        SpawnType::AtPosition { x, y },
    );
    if item_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: Don't know how to spawn [{}]!", name));
}
