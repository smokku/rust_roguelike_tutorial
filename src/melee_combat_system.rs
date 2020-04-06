use super::{CombatStats, Name, SufferDamage, WantsToMelee};
use legion::prelude::*;
use rltk::console;

pub fn build() -> std::boxed::Box<(dyn legion::systems::schedule::Schedulable + 'static)> {
    SystemBuilder::new("melee_combat")
        .with_query(<(Read<WantsToMelee>, Read<Name>, Read<CombatStats>)>::query())
        .read_component::<CombatStats>()
        .read_component::<Name>()
        .build(|command_buffer, world, _, query| {
            for (entity, (wants_melee, name, stats)) in query.iter_entities_mut(world) {
                let target = wants_melee.target;
                if stats.hp > 0 {
                    if let Some(target_stats) = world.get_component::<CombatStats>(target) {
                        let target_name = match world.get_component::<Name>(target) {
                            Some(name) => name.name.clone(),
                            None => "-Unnamed-".to_string(),
                        };
                        let damage = i32::max(0, stats.power - target_stats.defense);

                        if damage == 0 {
                            console::log(&format!(
                                "{} is unable to hurt {}",
                                &name.name, &target_name
                            ));
                        } else {
                            console::log(&format!(
                                "{} hits {}, for {} hp (of {}).",
                                &name.name, &target_name, damage, target_stats.hp
                            ));
                            SufferDamage::new_damage(&command_buffer, target, damage);
                            command_buffer.remove_component::<WantsToMelee>(entity);
                        }
                    } else {
                        console::log(&format!("{} does not do combat", target));
                    }
                } else {
                    console::log(&format!("{} is already dead, thus cannot do melee", entity))
                }
            }
        })
}
