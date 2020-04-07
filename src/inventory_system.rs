use super::{gamelog::GameLog, InBackpack, Name, Position, WantsToPickupItem};
use legion::prelude::*;

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
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
