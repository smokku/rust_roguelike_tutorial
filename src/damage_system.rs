use super::{
    gamelog::GameLog, Equipped, InBackpack, Map, Name, Player, Pools, Position, RunState,
    SufferDamage,
};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("damage")
        .with_query(Write::<SufferDamage>::query())
        .write_component::<Pools>()
        .read_component::<Position>()
        .write_resource::<Map>()
        .build(|command_buffer, world, map, query| unsafe {
            for (entity, mut damage) in query.iter_entities_unchecked(world) {
                if let Some(mut stats) = world.get_component_mut_unchecked::<Pools>(entity) {
                    stats.hit_points.current -= damage.amount.iter().sum::<i32>();
                }

                if let Some(pos) = world.get_component::<Position>(entity) {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.bloodstains.insert(idx);
                }

                damage.amount.clear();
                command_buffer.remove_component::<SufferDamage>(entity);
            }
        })
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
