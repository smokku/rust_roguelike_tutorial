use super::{
    gamelog::GameLog, particle_system::ParticleBuilder, CombatStats, DefenseBonus, Equipped,
    HungerClock, HungerState, MeleePowerBonus, Name, Position, SufferDamage, WantsToMelee,
};
use legion::prelude::*;
use rltk::console;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("melee_combat")
        .with_query(<(Read<WantsToMelee>, Read<Name>, Read<CombatStats>)>::query())
        .read_component::<CombatStats>()
        .read_component::<Name>()
        .read_component::<Position>()
        .read_component::<HungerClock>()
        .with_query(<(Read<MeleePowerBonus>, Read<Equipped>)>::query())
        .with_query(<(Read<DefenseBonus>, Read<Equipped>)>::query())
        .write_resource::<GameLog>()
        .write_resource::<ParticleBuilder>()
        .build(
            |command_buffer,
             world,
             (log, particle_builder),
             (query, query_melee, query_defense)| {
                for (entity, (wants_melee, name, stats)) in query.iter_entities(world) {
                    let target = wants_melee.target;
                    if stats.hp > 0 {
                        let mut offensive_bonus = 0;
                        for (power_bonus, equipped_by) in query_melee.iter(world) {
                            if equipped_by.owner == entity {
                                offensive_bonus += power_bonus.power;
                            }
                        }
                        if let Some(hc) = world.get_component::<HungerClock>(entity) {
                            if hc.state == HungerState::WellFed {
                                offensive_bonus += 1;
                            }
                        }

                        if let Some(target_stats) = world.get_component::<CombatStats>(target) {
                            let target_name = match world.get_component::<Name>(target) {
                                Some(name) => name.name.clone(),
                                None => "-Unnamed-".to_string(),
                            };

                            if let Some(pos) = world.get_component::<Position>(target) {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                            }

                            let mut defensive_bonus = 0;
                            for (defense_bonus, equipped_by) in query_defense.iter(&world) {
                                if equipped_by.owner == target {
                                    defensive_bonus += defense_bonus.defense;
                                }
                            }

                            let damage = i32::max(
                                0,
                                (stats.power + offensive_bonus)
                                    - (target_stats.defense + defensive_bonus),
                            );

                            if damage == 0 {
                                log.entries.push(format!(
                                    "{} is unable to hurt {}",
                                    &name.name, &target_name
                                ));
                            } else {
                                log.entries.push(format!(
                                    "{} hits {}, for {} hp.",
                                    &name.name, &target_name, damage
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
            },
        )
}
