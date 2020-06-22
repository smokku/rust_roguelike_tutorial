use super::Prefabs;
use crate::components::*;
use legion::prelude::*;
use std::collections::HashMap;

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
}
pub struct PrefabMaster {
    prefabs: Prefabs,
    item_index: HashMap<String, usize>,
}

impl PrefabMaster {
    pub fn empty() -> Self {
        PrefabMaster {
            prefabs: Prefabs { items: Vec::new() },
            item_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, prefabs: Prefabs) {
        self.prefabs = prefabs;
        self.item_index = HashMap::new();
        for (i, item) in self.prefabs.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }
}

pub fn spawn_named_item(
    prefabs: &PrefabMaster,
    world: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if prefabs.item_index.contains_key(key) {
        let item_template = &prefabs.prefabs.items[prefabs.item_index[key]];
        let entity = world.insert(
            (Item,),
            vec![(Name {
                name: item_template.name.clone(),
            },)],
        )[0];

        // Spawn in the specified location
        match pos {
            SpawnType::AtPosition { x, y } => {
                world
                    .add_component(entity, Position { x, y })
                    .expect("Cannot add component");
            }
        }

        // Renderable
        if let Some(renderable) = &item_template.renderable {
            world
                .add_component(
                    entity,
                    Renderable {
                        glyph: rltk::to_cp437(renderable.glyph),
                        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
                        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
                        render_order: renderable.order,
                    },
                )
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

        return Some(entity);
    }

    None
}
