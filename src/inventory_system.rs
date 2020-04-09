use super::{
    gamelog::GameLog, CombatStats, Consumable, InBackpack, InflictsDamage, Map, Name, Position,
    ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};
use legion::prelude::*;

pub fn build() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("item_collection")
        .with_query(Read::<WantsToPickupItem>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (player, gamelog), query| {
            // NOTE: In case of multiple requests to pick item up, the last one wins.
            // (As the InBackpack component gets overwritten)
            for (entity, pickup) in query.iter_entities(&world) {
                command_buffer.remove_component::<WantsToPickupItem>(entity);
                command_buffer.remove_component::<Position>(pickup.item);
                command_buffer.add_component(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                );

                if pickup.collected_by == **player {
                    let name = world.get_component::<Name>(pickup.item).unwrap();
                    gamelog
                        .entries
                        .push(format!("You pick up the {}.", name.name))
                }
            }
        })
}

pub fn item_use() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("item_use")
        .with_query(Read::<WantsToUseItem>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_resource::<Map>()
        .read_component::<Name>()
        .read_component::<InflictsDamage>()
        .read_component::<ProvidesHealing>()
        .build(|command_buffer, world, (player, gamelog, map), query| {
            for (entity, use_item) in query.iter_entities(&world) {
                let player_entity = **player;
                let item_entity = use_item.item;
                let mut used_item = false;

                let item_name = if let Some(item_name) = world.get_component::<Name>(item_entity) {
                    item_name.name.clone()
                } else {
                    "-Unknown-".to_string()
                };

                // If it heals, apply the healing
                if let Some(healer) = world.get_component::<ProvidesHealing>(item_entity) {
                    let heal_amount = healer.heal_amount;
                    command_buffer.exec_mut(move |world| {
                        let mut stats = world
                            .get_component_mut::<CombatStats>(player_entity)
                            .unwrap();
                        stats.hp = i32::min(stats.max_hp, stats.hp + heal_amount);
                    });

                    if entity == player_entity {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            item_name, healer.heal_amount
                        ));
                    }

                    used_item = true;
                }

                // If it inflicts damage, apply damage to all target cell entities
                if let Some(damages) = world.get_component::<InflictsDamage>(item_entity) {
                    let target_point = use_item.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    for mob in map.tile_content[idx].iter() {
                        let mob_entity = *mob;
                        let damage = damages.damage;
                        SufferDamage::new_damage(command_buffer, *mob, damage);

                        if entity == player_entity {
                            let mob_name =
                                if let Some(mob_name) = world.get_component::<Name>(mob_entity) {
                                    mob_name.name.clone()
                                } else {
                                    "-Unnamed-".to_string()
                                };

                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name, mob_name, damage
                            ));
                        }

                        used_item = true;
                    }
                }

                // If it's a consumable, we delete it on use
                if used_item {
                    if let Some(_consumable) = world.get_tag::<Consumable>(item_entity) {
                        command_buffer.delete(item_entity);
                    }
                }
                command_buffer.remove_component::<WantsToUseItem>(entity);
            }
        })
}

pub fn item_drop() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("item_drop")
        .with_query(<(Read<WantsToDropItem>, Read<Position>)>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (player, gamelog), query| {
            for (entity, (to_drop, dropper_pos)) in query.iter_entities(&world) {
                let item_entity = to_drop.item;
                command_buffer.remove_component::<InBackpack>(item_entity);
                command_buffer.add_component(item_entity, *dropper_pos);

                let item_name = if let Some(item_name) = world.get_component::<Name>(item_entity) {
                    item_name.name.clone()
                } else {
                    "-Unknown-".to_string()
                };
                if entity == **player {
                    gamelog.entries.push(format!("You drop the {}.", item_name));
                }
                command_buffer.remove_component::<WantsToDropItem>(entity);
            }
        })
}
