use super::{gamelog::GameLog, CombatStats, Name, Player, RunState, SufferDamage};
use legion::prelude::*;

pub fn build() -> Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("damage")
        .with_query(Write::<SufferDamage>::query())
        .write_component::<CombatStats>()
        .build(|command_buffer, world, _, query| {
            for (entity, mut damage) in query.iter_entities_mut(world) {
                if let Some(mut stats) = world.get_component_mut::<CombatStats>(entity) {
                    stats.hp -= damage.amount.iter().sum::<i32>();
                }
                damage.amount.clear();
                command_buffer.remove_component::<SufferDamage>(entity);
            }
        })
}

pub fn delete_the_dead(world: &mut World, resources: &mut Resources) {
    let mut dead = Vec::new();
    let query = Read::<CombatStats>::query();
    for (victim, stats) in query.iter_entities(world) {
        if stats.hp < 1 {
            if let Some(_player) = world.get_tag::<Player>(victim) {
                resources.insert(RunState::GameOver);
            } else {
                dead.push(victim);
            }
        }
    }

    let mut log = resources.get_mut::<GameLog>().unwrap();
    for victim in dead {
        let name = if let Some(name) = world.get_component::<Name>(victim) {
            name.name.clone()
        } else {
            "-Unnamed-".to_string()
        };
        log.entries
            .push(format!("{} is pushing up the daisies.", name));
        world.delete(victim);
    }
}
