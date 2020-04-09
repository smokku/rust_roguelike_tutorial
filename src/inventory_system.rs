use super::{
    gamelog::GameLog, AreaOfEffect, CombatStats, Consumable, InBackpack, InflictsDamage, Map, Name,
    Position, ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
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
        .read_component::<AreaOfEffect>()
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

                // Targeting
                let mut targets = Vec::new();
                match use_item.target {
                    None => targets.push(player_entity),
                    Some(target) => {
                        let area_effect = world.get_component::<AreaOfEffect>(item_entity);
                        let mut target_tiles = Vec::new();
                        match area_effect {
                            None => {
                                // Single tile target
                                target_tiles.push(target);
                            }
                            Some(area_effect) => {
                                // AoE
                                target_tiles =
                                    rltk::field_of_view(target, area_effect.radius, &**map);
                                target_tiles.retain(|p| {
                                    p.x > 0
                                        && p.x < map.width - 1
                                        && p.y > 0
                                        && p.y < map.height - 1
                                });
                            }
                        }
                        for tile in target_tiles.iter() {
                            let idx = map.xy_idx(tile.x, tile.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                    }
                }

                // If it heals, apply the healing
                if let Some(healer) = world.get_component::<ProvidesHealing>(item_entity) {
                    let heal_amount = healer.heal_amount;
                    for target in targets.iter() {
                        let target = *target;
                        command_buffer.exec_mut(move |world| {
                            let stats = world.get_component_mut::<CombatStats>(target);
                            if let Some(mut stats) = stats {
                                stats.hp = i32::min(stats.max_hp, stats.hp + heal_amount);
                            }
                        });
                        if target == player_entity {
                            gamelog.entries.push(format!(
                                "You drink the {}, healing {} hp.",
                                item_name, heal_amount
                            ));
                        }
                    }
                    used_item = true;
                }

                // If it inflicts damage, apply damage to all target cell entities
                if let Some(damages) = world.get_component::<InflictsDamage>(item_entity) {
                    for mob in targets.iter() {
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
