use super::{
    gamelog::GameLog, CombatStats, InBackpack, Name, Position, Potion, WantsToDrinkPotion,
    WantsToDropItem, WantsToPickupItem,
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

pub fn potion_use() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("potion_use")
        .with_query(<(Read<WantsToDrinkPotion>, Write<CombatStats>)>::query())
        .read_resource::<Entity>()
        .write_resource::<GameLog>()
        .read_component::<Potion>()
        .read_component::<Name>()
        .build(|command_buffer, mut world, (player, gamelog), query| {
            for (entity, (wants_drink, mut stats)) in query.iter_entities_mut(&mut world) {
                let potion_entity = wants_drink.potion;
                if let Some(potion) = world.get_component::<Potion>(potion_entity) {
                    stats.hp = i32::min(stats.max_hp, stats.hp + potion.heal_amount);
                    let potion_name =
                        if let Some(potion_name) = world.get_component::<Name>(potion_entity) {
                            potion_name.name.clone()
                        } else {
                            "-Unknown-".to_string()
                        };
                    if entity == **player {
                        gamelog.entries.push(format!(
                            "You drink the {}, healing {} hp.",
                            potion_name, potion.heal_amount
                        ));
                    }
                    command_buffer.delete(potion_entity);
                }
                command_buffer.remove_component::<WantsToDrinkPotion>(entity);
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
