use super::{CombatStats, Name, SufferDamage};
use legion::prelude::*;
use rltk::console;

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("damage")
        .with_query(<(Write<CombatStats>, Write<SufferDamage>)>::query())
        .build(|command_buffer, world, _, query| {
            for (entity, (mut stats, mut damage)) in query.iter_entities_mut(world) {
                stats.hp -= damage.amount.iter().sum::<i32>();
                damage.amount.clear();
                command_buffer.remove_component::<SufferDamage>(entity);
            }
        })
}

pub fn delete_the_dead() -> Box<dyn Fn(&mut World, &mut Resources) -> ()> {
    Box::new(|world: &mut World, resources: &mut Resources| {
        let mut dead = Vec::new();
        let query = Read::<CombatStats>::query();
        for (entity, stats) in query.iter_entities(world) {
            if stats.hp < 1 {
                let player = resources.get::<Entity>().expect("Cannot get Player entity");
                if entity == *player {
                    console::log("You are dead");
                } else {
                    dead.push(entity);
                }
            }
        }
        for victim in dead {
            let name = if let Some(name) = world.get_component::<Name>(victim) {
                name.name.clone()
            } else {
                "-Unnamed-".to_string()
            };
            console::log(format!("{} is pushing up the daisies.", name));
            world.delete(victim);
        }
    })
}
