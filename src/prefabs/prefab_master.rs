use super::{Prefabs, SpawnTableEntry};
use crate::{attr_bonus, components::*, mana_at_level, npc_hp, random_table::RandomTable};
use legion::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};

const BASE_ATTRIBUTE: i32 = 11;

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex = Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").unwrap();
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for cap in DICE_RE.captures_iter(dice) {
        if let Some(group) = cap.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = cap.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}
pub struct PrefabMaster {
    prefabs: Prefabs,
    item_index: HashMap<String, usize>,
    mob_index: HashMap<String, usize>,
    prop_index: HashMap<String, usize>,
}

impl PrefabMaster {
    pub fn empty() -> Self {
        PrefabMaster {
            prefabs: Prefabs {
                spawn_table: Vec::new(),
                items: Vec::new(),
                mobs: Vec::new(),
                props: Vec::new(),
            },
            item_index: HashMap::new(),
            mob_index: HashMap::new(),
            prop_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, prefabs: Prefabs) {
        self.prefabs = prefabs;
        self.item_index = HashMap::new();

        let mut used_names = HashSet::<String>::new();
        for (i, item) in self.prefabs.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate item name in prefabs [{}]",
                    item.name
                ));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, mob) in self.prefabs.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate mob name in prefabs [{}]",
                    mob.name
                ));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, prop) in self.prefabs.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!(
                    "WARNING - duplicate prop name in prefabs [{}]",
                    prop.name
                ));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.prefabs.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING - Spawn tables references unspecified entity [{}]",
                    spawn.name
                ));
            }
        }
    }
}

fn find_slot_for_equippable_item(tag: &str, pm: &PrefabMaster) -> EquipmentSlot {
    if !pm.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = pm.item_index[tag];
    let item = &pm.prefabs.items[item_index];
    if let Some(wpn) = &item.weapon {
        return match wpn.range.to_lowercase().as_str() {
            "melee" => EquipmentSlot::Melee,
            range => {
                rltk::console::log(format!("Warning: unknown weapon range type [{}])", range));
                EquipmentSlot::Melee
            }
        };
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

fn spawn_position(world: &mut World, entity: Entity, pos: SpawnType, tag: &str, pm: &PrefabMaster) {
    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => {
            world
                .add_component(entity, Position { x, y })
                .expect("Cannot add component");
        }
        SpawnType::Carried { by } => {
            world
                .add_component(entity, InBackpack { owner: by })
                .expect("Cannot add component");
        }
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, pm);
            world
                .add_component(entity, Equipped { owner: by, slot })
                .expect("Cannot add component");
        }
    }
}

fn get_renderable_component(renderable: &super::item_structs::Renderable) -> Renderable {
    Renderable {
        glyph: rltk::to_cp437(renderable.glyph),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn spawn_named_entity(
    pm: &PrefabMaster,
    world: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if pm.item_index.contains_key(key) {
        return spawn_named_item(pm, world, key, pos);
    } else if pm.mob_index.contains_key(key) {
        return spawn_named_mob(pm, world, key, pos);
    } else if pm.prop_index.contains_key(key) {
        return spawn_named_prop(pm, world, key, pos);
    }

    rltk::console::log(format!(
        "Warning: attempt to spawn unknown entity: [{}]",
        key
    ));

    None
}

pub fn spawn_named_item(
    pm: &PrefabMaster,
    world: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if pm.item_index.contains_key(key) {
        let item_template = &pm.prefabs.items[pm.item_index[key]];
        let entity = world.insert(
            (Item,),
            vec![(Name {
                name: item_template.name.clone(),
            },)],
        )[0];

        spawn_position(world, entity, pos, key, pm);

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            world
                .add_component(entity, get_renderable_component(renderable))
                .expect("Cannot add component");
        }

        if let Some(consumable) = &item_template.consumable {
            world
                .add_tag(entity, Consumable {})
                .expect("Cannot add tag");

            for (effect, value) in consumable.effects.iter() {
                match effect.as_str() {
                    "provides_healing" => {
                        world
                            .add_component(
                                entity,
                                ProvidesHealing {
                                    heal_amount: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "ranged" => {
                        world
                            .add_component(
                                entity,
                                Ranged {
                                    range: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "damage" => {
                        world
                            .add_component(
                                entity,
                                InflictsDamage {
                                    damage: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "area_of_effect" => {
                        world
                            .add_component(
                                entity,
                                AreaOfEffect {
                                    radius: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "confusion" => {
                        world
                            .add_component(
                                entity,
                                Confusion {
                                    turns: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "magic_mapping" => {
                        world
                            .add_tag(entity, MagicMapper {})
                            .expect("Cannot add tag");
                    }
                    "food" => {
                        world
                            .add_tag(entity, ProvidesFood {})
                            .expect("Cannot add tag");
                    }
                    effect_name => {
                        rltk::console::log(format!(
                            "Warning: consumable effect {} not implemented.",
                            effect_name
                        ));
                    }
                }
            }
        }

        // Weapon
        if let Some(weapon) = &item_template.weapon {
            match weapon.range.as_str() {
                "melee" => {
                    world
                        .add_component(
                            entity,
                            Equippable {
                                slot: EquipmentSlot::Melee,
                            },
                        )
                        .expect("Cannot add component");
                    let (n_dice, die_type, die_bonus) = parse_dice_string(&weapon.base_damage);
                    let mut wpn = MeleeWeapon {
                        attribute: WeaponAttribute::Might,
                        damage_n_dice: n_dice,
                        damage_die_type: die_type,
                        damage_bonus: die_bonus,
                        hit_bonus: weapon.hit_bonus,
                    };
                    match weapon.attribute.to_lowercase().as_str() {
                        "quickness" => {
                            wpn.attribute = WeaponAttribute::Quickness;
                        }
                        "might" => {
                            wpn.attribute = WeaponAttribute::Might;
                        }
                        _ => {
                            rltk::console::log(format!(
                                "Warning: weapon attribute {} not implemented.",
                                weapon.attribute
                            ));
                        }
                    }
                    world
                        .add_component(entity, wpn)
                        .expect("Cannot add component");
                }
                range_type => {
                    rltk::console::log(format!(
                        "Warning: range type {} not implemented.",
                        range_type
                    ));
                }
            }
        }

        // Shield
        if let Some(shield) = &item_template.shield {
            world
                .add_component(
                    entity,
                    Equippable {
                        slot: EquipmentSlot::Shield,
                    },
                )
                .expect("Cannot add component");
            world
                .add_component(
                    entity,
                    DefenseBonus {
                        defense: shield.defense_bonus,
                    },
                )
                .expect("Cannot add component");
        }

        return Some(entity);
    }

    None
}

pub fn spawn_named_mob(
    pm: &PrefabMaster,
    world: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if pm.mob_index.contains_key(key) {
        let mob_template = &pm.prefabs.mobs[pm.mob_index[key]];
        let entity = world.insert(
            (),
            vec![(
                Name {
                    name: mob_template.name.clone(),
                },
                Viewshed {
                    visible_tiles: Vec::new(),
                    range: mob_template.vision_range,
                    dirty: true,
                },
            )],
        )[0];

        spawn_position(world, entity, pos, key, pm);

        // AI Type
        match mob_template.ai.as_str() {
            "melee" => world.add_tag(entity, Monster {}).expect("Cannot add tag"),
            "bystander" => world.add_tag(entity, Bystander {}).expect("Cannot add tag"),
            "vendor" => world.add_tag(entity, Vendor {}).expect("Cannot add tag"),
            ai_type => {
                rltk::console::log(format!("Warning: AI type {} not implemented.", ai_type));
            }
        }

        if let Some(renderable) = &mob_template.renderable {
            world
                .add_component(entity, get_renderable_component(renderable))
                .expect("Cannot add component");
        }

        if let Some(quips) = &mob_template.quips {
            world
                .add_component(
                    entity,
                    Quips {
                        available: quips.clone(),
                    },
                )
                .expect("Cannot add component");
        }

        let mut attr = Attributes {
            might: Attribute {
                base: BASE_ATTRIBUTE,
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
        };
        if let Some(might) = mob_template.attributes.might {
            attr.might = Attribute {
                base: might,
                modifiers: 0,
                bonus: attr_bonus(might),
            };
        }
        if let Some(fitness) = mob_template.attributes.fitness {
            attr.fitness = Attribute {
                base: fitness,
                modifiers: 0,
                bonus: attr_bonus(fitness),
            };
        }
        if let Some(quickness) = mob_template.attributes.quickness {
            attr.quickness = Attribute {
                base: quickness,
                modifiers: 0,
                bonus: attr_bonus(quickness),
            };
        }
        if let Some(intelligence) = mob_template.attributes.intelligence {
            attr.intelligence = Attribute {
                base: intelligence,
                modifiers: 0,
                bonus: attr_bonus(intelligence),
            };
        }

        let level = if mob_template.level.is_some() {
            mob_template.level.unwrap()
        } else {
            1
        };
        let hit_points = npc_hp(attr.fitness.base, level);
        let mana = mana_at_level(attr.intelligence.base, level);

        world
            .add_component(entity, attr)
            .expect("Cannot add component");

        world
            .add_component(
                entity,
                Pools {
                    level,
                    experience: 0,
                    hit_points: Pool {
                        current: hit_points,
                        max: hit_points,
                    },
                    mana: Pool {
                        current: mana,
                        max: mana,
                    },
                },
            )
            .expect("Cannot add component");

        let mut skills = Skills {
            skills: HashMap::new(),
        };
        skills.skills.insert(Skill::Melee, 1);
        skills.skills.insert(Skill::Defense, 1);
        skills.skills.insert(Skill::Magic, 1);
        if let Some(mobskills) = &mob_template.skills {
            for sk in mobskills.iter() {
                match sk.0.to_lowercase().as_str() {
                    "melee" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    "defense" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    "magic" => {
                        skills.skills.insert(Skill::Melee, *sk.1);
                    }
                    _ => {
                        rltk::console::log(format!("Unknown skill referenced: \"{}\"", sk.0));
                    }
                }
            }
        }
        world
            .add_component(entity, skills)
            .expect("Cannot add component");

        if mob_template.blocks_tile {
            world
                .add_tag(entity, BlocksTile {})
                .expect("Cannot add tag");
        }

        return Some(entity);
    }

    None
}

pub fn spawn_named_prop(
    pm: &PrefabMaster,
    world: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if pm.prop_index.contains_key(key) {
        let prop_template = &pm.prefabs.props[pm.prop_index[key]];
        let entity = world.insert(
            (Monster,),
            vec![(Name {
                name: prop_template.name.clone(),
            },)],
        )[0];

        spawn_position(world, entity, pos, key, pm);

        // Renderable
        if let Some(renderable) = &prop_template.renderable {
            world
                .add_component(entity, get_renderable_component(renderable))
                .expect("Cannot add component");
        }

        if let Some(hidden) = prop_template.hidden {
            if hidden {
                world.add_tag(entity, Hidden {}).expect("Cannot add tag");
            }
        }
        if let Some(blocks_tile) = prop_template.blocks_tile {
            if blocks_tile {
                world
                    .add_tag(entity, BlocksTile {})
                    .expect("Cannot add tag");
            }
        }
        if let Some(blocks_visibility) = prop_template.blocks_visibility {
            if blocks_visibility {
                world
                    .add_tag(entity, BlocksVisibility {})
                    .expect("Cannot add tag");
            }
        }
        if let Some(door_open) = prop_template.door_open {
            world
                .add_component(entity, Door { open: door_open })
                .expect("Cannot add component");
        }
        if let Some(entry_trigger) = &prop_template.entry_trigger {
            world
                .add_tag(entity, EntryTrigger {})
                .expect("Cannot add tag");
            for (effect, value) in entry_trigger.effects.iter() {
                match effect.as_str() {
                    "damage" => {
                        world
                            .add_component(
                                entity,
                                InflictsDamage {
                                    damage: value.parse().unwrap(),
                                },
                            )
                            .expect("Cannot add component");
                    }
                    "single_activation" => {
                        world
                            .add_tag(entity, SingleActivation {})
                            .expect("Cannot add tag");
                    }
                    effect_name => {
                        rltk::console::log(format!(
                            "Warning: consumable effect {} not implemented.",
                            effect_name
                        ));
                    }
                }
            }
        }

        return Some(entity);
    }

    None
}

pub fn get_spawn_table_for_depth(pm: &PrefabMaster, depth: i32) -> RandomTable {
    let available_options: Vec<&SpawnTableEntry> = pm
        .prefabs
        .spawn_table
        .iter()
        .filter(|entry| entry.min_depth <= depth && depth <= entry.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for entry in available_options.iter() {
        let mut weight = entry.weight;
        if entry.add_map_depth_to_weight.is_some() {
            weight += depth;
        }
        rt = rt.add(entry.name.clone(), weight);
    }

    rt
}
