use super::{
    gamelog::GameLog, mana_at_level, player_hp_at_level, Attributes, Equipped, InBackpack,
    LootTable, Map, Name, Player, Pools, Position, RunState, SufferDamage,
};
use crate::prefabs::{get_item_drop, spawn_named_item, SpawnType, PREFABS};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("damage")
        .with_query(Write::<SufferDamage>::query())
        .write_component::<Pools>()
        .read_component::<Position>()
        .write_resource::<Map>()
        .read_resource::<Entity>()
        .read_component::<Attributes>()
        .build(
            |command_buffer, world, (map, player_entity), query| unsafe {
                let mut xp_gain = 0;
                for (entity, mut damage) in query.iter_entities_unchecked(world) {
                    if let Some(mut stats) = world.get_component_mut_unchecked::<Pools>(entity) {
                        for (dmg, from_player) in damage.amount.iter() {
                            stats.hit_points.current -= dmg;

                            if stats.hit_points.current < 1 && *from_player {
                                xp_gain += stats.level * 100;
                            }
                        }

                        if let Some(pos) = world.get_component::<Position>(entity) {
                            let idx = map.xy_idx(pos.x, pos.y);
                            map.bloodstains.insert(idx);
                        }
                    }

                    damage.amount.clear();
                    command_buffer.remove_component::<SufferDamage>(entity);
                }

                if xp_gain != 0 {
                    let player_attributes =
                        *(world.get_component::<Attributes>(**player_entity).unwrap());
                    let mut player_stats =
                        world.get_component_mut::<Pools>(**player_entity).unwrap();
                    player_stats.experience += xp_gain;
                    if player_stats.experience >= player_stats.level * 1000 {
                        // We've gone up a level!
                        player_stats.level = player_stats.experience / 1000 + 1;
                        player_stats.hit_points.max = player_hp_at_level(
                            player_attributes.fitness.base + player_attributes.fitness.modifiers,
                            player_stats.level,
                        );
                        player_stats.hit_points.current = player_stats.hit_points.max;
                        player_stats.mana.max = mana_at_level(
                            player_attributes.intelligence.base
                                + player_attributes.intelligence.modifiers,
                            player_stats.level,
                        );
                        player_stats.mana.current = player_stats.mana.max;
                    }
                }
            },
        )
}

pub fn delete_the_dead(world: &mut World, resources: &mut Resources) {
    let mut dead = Vec::new();
    for (victim, stats) in Read::<Pools>::query().iter_entities(world) {
        if stats.hit_points.current < 1 {
            if let Some(_player) = world.get_tag::<Player>(victim) {
                resources.insert(RunState::GameOver);
            } else {
                dead.push(victim);
            }
        }
    }

    // Drop everything held by dead people
    let mut to_drop = Vec::new();
    let mut to_spawn = Vec::new();
    for victim in dead.iter() {
        if let Some(pos) = world.get_component::<Position>(*victim) {
            // Drop their stuff
            for (entity, equipped) in Read::<Equipped>::query().iter_entities(world) {
                if equipped.owner == *victim {
                    to_drop.push((entity, (*pos).clone()));
                }
            }
            for (entity, backpack) in Read::<InBackpack>::query().iter_entities(world) {
                if backpack.owner == *victim {
                    to_drop.push((entity, (*pos).clone()));
                }
            }

            if let Some(loot) = world.get_component::<LootTable>(*victim) {
                let mut rng = resources.get_mut::<rltk::RandomNumberGenerator>().unwrap();
                if let Some(drop) = get_item_drop(&PREFABS.lock().unwrap(), &mut rng, &loot.table) {
                    to_spawn.push((drop, (*pos).clone()));
                }
            }
        }
    }
    for (entity, pos) in to_drop.drain(..) {
        world
            .remove_components::<(Equipped, InBackpack)>(entity)
            .expect("Dropping item failed");
        world
            .add_component(entity, pos)
            .expect("Positioning dropped item failed");
    }

    for (tag, pos) in to_spawn.drain(..) {
        spawn_named_item(
            &PREFABS.lock().unwrap(),
            world,
            &tag,
            SpawnType::AtPosition { x: pos.x, y: pos.y },
        );
    }

    let mut log = resources.get_mut::<GameLog>().unwrap();
    for victim in dead.iter() {
        let name = if let Some(name) = world.get_component::<Name>(*victim) {
            name.name.clone()
        } else {
            "-Unnamed-".to_string()
        };
        log.entries
            .push(format!("{} is pushing up the daisies.", name));
        world.delete(*victim);
    }
}
