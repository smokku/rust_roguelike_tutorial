use super::{
    components::*, field_of_view, gamelog::GameLog, particle_system::ParticleBuilder, Map, RunState,
};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("item_collection")
        .with_query(Read::<WantsToPickupItem>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (player, gamelog), query| {
            // NOTE: In case of multiple requests to pick item up, the last one wins.
            // (As the InBackpack component gets overwritten)
            for (entity, pickup) in query.iter_entities(world) {
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

pub fn item_use() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("item_use")
        .with_query(Read::<WantsToUseItem>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_resource::<Map>()
        .write_resource::<ParticleBuilder>()
        .read_component::<Name>()
        .read_component::<AreaOfEffect>()
        .read_component::<InflictsDamage>()
        .read_component::<ProvidesHealing>()
        .read_component::<Confusion>()
        .read_component::<Equippable>()
        .read_component::<Position>()
        .with_query(<(Read<Equipped>, Read<Name>)>::query())
        .write_resource::<RunState>()
        .build(
            #[allow(clippy::cognitive_complexity)]
            |command_buffer,
             world,
             (player, gamelog, map, particle_builder, runstate),
             (query, query_equipped)| {
                for (entity, use_item) in query.iter_entities(world) {
                    let player_entity = **player;
                    let item_entity = use_item.item;
                    let mut used_item = false;

                    let item_name =
                        if let Some(item_name) = world.get_component::<Name>(item_entity) {
                            item_name.name.clone()
                        } else {
                            "-Unknown-".to_string()
                        };

                    // Targeting
                    let mut targets = Vec::new();
                    match use_item.target {
                        None => targets.push((player_entity, "Player".to_string())),
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
                                        field_of_view(target, area_effect.radius as usize, &**map);
                                }
                            }
                            for tile in target_tiles.iter() {
                                let idx = map.xy_idx(tile.x, tile.y);
                                for mob in map.tile_content[idx].iter() {
                                    let mob_entity = *mob;
                                    let mob_name = if let Some(mob_name) =
                                        world.get_component::<Name>(mob_entity)
                                    {
                                        mob_name.name.clone()
                                    } else {
                                        "-Unnamed-".to_string()
                                    };
                                    targets.push((mob_entity, mob_name));
                                }
                                particle_builder.request(
                                    tile.x,
                                    tile.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }

                    // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
                    if let Some(can_equip) = world.get_component::<Equippable>(item_entity) {
                        let target_slot = can_equip.slot;
                        let target = targets[0].0;

                        // Remove any items the target has in the item's slot
                        let mut to_unequip = Vec::new();
                        for (item_entity, (already_equipped, name)) in
                            query_equipped.iter_entities(world)
                        {
                            if already_equipped.owner == target
                                && already_equipped.slot == target_slot
                            {
                                to_unequip.push(item_entity);
                                if target == player_entity {
                                    gamelog.entries.push(format!("You unequip {}.", name.name));
                                }
                            }
                        }
                        for item in to_unequip.iter() {
                            command_buffer.remove_component::<Equipped>(*item);
                            command_buffer.add_component(*item, InBackpack { owner: target });
                        }

                        // Wield the item
                        command_buffer.add_component(
                            item_entity,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        );
                        command_buffer.remove_component::<InBackpack>(item_entity);
                        if target == player_entity {
                            let name = world.get_component::<Name>(item_entity).unwrap();
                            gamelog
                                .entries
                                .push(format!("You equip the {}.", name.name));
                        }
                    }

                    // It it is edible, eat it!
                    if let Some(_food) = world.get_tag::<ProvidesFood>(item_entity) {
                        let target = targets[0].0;
                        command_buffer.exec_mut(move |world| {
                            if let Some(mut hc) = world.get_component_mut::<HungerClock>(target) {
                                hc.state = HungerState::WellFed;
                                hc.duration = 20;
                            }
                        });
                        if target == player_entity {
                            let name = world.get_component::<Name>(item_entity).unwrap();
                            gamelog.entries.push(format!("You eat the {}.", name.name));
                        }
                        used_item = true;
                    }

                    // It it's a magic mapper...
                    if let Some(_mm) = world.get_tag::<MagicMapper>(item_entity) {
                        gamelog
                            .entries
                            .push("The map is revealed to you!".to_string());
                        **runstate = RunState::MagicMapReveal { row: 0 };
                        used_item = true;
                    }

                    // If it heals, apply the healing
                    if let Some(healer) = world.get_component::<ProvidesHealing>(item_entity) {
                        let heal_amount = healer.heal_amount;
                        for (target_entity, _target_name) in targets.iter() {
                            let target_entity = *target_entity;
                            command_buffer.exec_mut(move |world| {
                                if let Some(mut stats) =
                                    world.get_component_mut::<Pools>(target_entity)
                                {
                                    stats.hit_points.current = i32::min(
                                        stats.hit_points.max,
                                        stats.hit_points.current + heal_amount,
                                    );
                                }
                            });
                            if let Some(pos) = world.get_component::<Position>(target_entity) {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::GREEN),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('♥'),
                                    200.0,
                                );
                            }

                            if entity == player_entity {
                                gamelog.entries.push(format!(
                                    "You use the {}, healing {} hp.",
                                    item_name, heal_amount
                                ));
                            }
                        }
                        used_item = true;
                    }

                    // If it inflicts damage, apply damage to all target cell entities
                    if let Some(damages) = world.get_component::<InflictsDamage>(item_entity) {
                        for (target_entity, target_name) in targets.iter() {
                            let target_entity = *target_entity;
                            let damage = damages.damage;
                            SufferDamage::new_damage(command_buffer, target_entity, damage);
                            if let Some(pos) = world.get_component::<Position>(target_entity) {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                            }

                            if entity == player_entity {
                                gamelog.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name, target_name, damage
                                ));
                            }
                        }
                        used_item = true;
                    }

                    // Can it pass along confusion?
                    if let Some(confusion) = world.get_component::<Confusion>(item_entity) {
                        for (target_entity, target_name) in targets.iter() {
                            let target_entity = *target_entity;
                            command_buffer.add_component(target_entity, *confusion);
                            if let Some(pos) = world.get_component::<Position>(target_entity) {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::MAGENTA),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('?'),
                                    200.0,
                                );
                            }

                            if entity == player_entity {
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name, target_name
                                ));
                            }
                        }
                        used_item = true;
                    }

                    // If it's a consumable, we delete it on use
                    if used_item {
                        if let Some(_consumable) = world.get_tag::<Consumable>(item_entity) {
                            command_buffer.delete(item_entity);
                        }
                    }
                    command_buffer.remove_component::<WantsToUseItem>(entity);
                }
            },
        )
}

pub fn item_drop() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("item_drop")
        .with_query(<(Read<WantsToDropItem>, Read<Position>)>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (player, gamelog), query| {
            for (entity, (to_drop, dropper_pos)) in query.iter_entities(world) {
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

pub fn item_remove() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("item_remove")
        .with_query(Read::<WantsToRemoveItem>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Name>()
        .build(|command_buffer, world, (player, gamelog), query| {
            for (entity, to_remove) in query.iter_entities(world) {
                let item_entity = to_remove.item;
                command_buffer.remove_component::<Equipped>(item_entity);
                command_buffer.add_component(item_entity, InBackpack { owner: entity });

                let item_name = if let Some(item_name) = world.get_component::<Name>(item_entity) {
                    item_name.name.clone()
                } else {
                    "-Unknown-".to_string()
                };
                if entity == **player {
                    gamelog
                        .entries
                        .push(format!("You remove the {}.", item_name));
                }
                command_buffer.remove_component::<WantsToRemoveItem>(entity);
            }
        })
}

pub fn activate_item(world: &mut World, resources: &Resources, item: Entity) -> RunState {
    if let Some(ranged) = world.get_component::<Ranged>(item) {
        return RunState::ShowTargeting {
            range: ranged.range,
            item,
        };
    }

    world
        .add_component(
            *resources.get::<Entity>().unwrap(),
            WantsToUseItem { item, target: None },
        )
        .expect("Unable to insert intent");

    return RunState::PlayerTurn;
}
