use super::{components::*, gamelog::GameLog, particle_system::ParticleBuilder, skill_bonus};
use legion::prelude::*;
use rltk::{console, RandomNumberGenerator};

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("melee_combat")
        .with_query(<(
            Read<WantsToMelee>,
            Read<Attributes>,
            Read<Skills>,
            Read<Pools>,
        )>::query())
        .read_component::<Name>()
        .read_component::<Position>()
        .read_component::<HungerClock>()
        .with_query(<(Read<MeleeWeapon>, Read<Equipped>)>::query())
        .with_query(<(Read<Wearable>, Read<Equipped>)>::query())
        .write_resource::<GameLog>()
        .write_resource::<ParticleBuilder>()
        .write_resource::<RandomNumberGenerator>()
        .build(
            |command_buffer,
             world,
             (log, particle_builder, rng),
             (query, query_melee, query_defense)| {
                for (entity, (wants_melee, attacker_attributes, attacker_skills, attacker_pools)) in
                    query.iter_entities(world)
                {
                    command_buffer.remove_component::<WantsToMelee>(entity);

                    let target = wants_melee.target;
                    let target_attributes = world.get_component::<Attributes>(target);
                    let target_skills = world.get_component::<Skills>(target);
                    let target_pools = world.get_component::<Pools>(target);

                    let attacker_name = match world.get_component::<Name>(entity) {
                        Some(name) => name.name.clone(),
                        None => "-Unnamed-".to_string(),
                    };
                    let target_name = match world.get_component::<Name>(target) {
                        Some(name) => name.name.clone(),
                        None => "-Unnamed-".to_string(),
                    };

                    match (target_attributes, target_skills, target_pools) {
                        (Some(target_attributes), Some(target_skills), Some(target_pools)) => {
                            // Are the attacker and defender alive? Only attack if they are
                            if attacker_pools.hit_points.current > 0
                                && target_pools.hit_points.current > 0
                            {
                                let mut weapon_info = MeleeWeapon {
                                    attribute: WeaponAttribute::Might,
                                    hit_bonus: 0,
                                    damage_n_dice: 1,
                                    damage_die_type: 4,
                                    damage_bonus: 0,
                                };

                                for (melee, wielded) in query_melee.iter(world) {
                                    if wielded.owner == entity
                                        && wielded.slot == EquipmentSlot::Melee
                                    {
                                        weapon_info = *melee;
                                    }
                                }

                                let natural_roll = rng.roll_dice(1, 20);
                                let attribute_hit_bonus = match weapon_info.attribute {
                                    WeaponAttribute::Might => attacker_attributes.might.bonus,
                                    WeaponAttribute::Quickness => {
                                        attacker_attributes.quickness.bonus
                                    }
                                };
                                let skill_hit_bonus = skill_bonus(Skill::Melee, &*attacker_skills);
                                let weapon_hit_bonus = weapon_info.hit_bonus;
                                let mut status_hit_bonus = 0;
                                if let Some(hc) = world.get_component::<HungerClock>(entity) {
                                    if hc.state == HungerState::WellFed {
                                        status_hit_bonus += 1;
                                    }
                                }
                                let modified_hit_roll = natural_roll
                                    + attribute_hit_bonus
                                    + skill_hit_bonus
                                    + weapon_hit_bonus
                                    + status_hit_bonus;

                                let mut armor_item_bonus_f = 0.0;
                                for (armor, wielded) in query_defense.iter(world) {
                                    if wielded.owner == wants_melee.target {
                                        armor_item_bonus_f += armor.armor_class;
                                    }
                                }

                                let base_armor_class = 10;
                                let armor_quickness_bonus = target_attributes.quickness.bonus;
                                let armor_skill_bonus =
                                    skill_bonus(Skill::Defense, &*target_skills);
                                let armor_item_bonus = armor_item_bonus_f as i32;
                                let armor_class = base_armor_class
                                    + armor_quickness_bonus
                                    + armor_skill_bonus
                                    + armor_item_bonus;

                                if natural_roll == 1 {
                                    // Natural 1 miss
                                    log.entries.push(format!(
                                        "{} considers attacking {}, but misjudges the timing.",
                                        attacker_name, target_name
                                    ));
                                    if let Some(pos) = world.get_component::<Position>(target) {
                                        particle_builder.request(
                                            pos.x,
                                            pos.y,
                                            rltk::RGB::named(rltk::BLUE),
                                            rltk::RGB::named(rltk::BLACK),
                                            rltk::to_cp437('‼'),
                                            200.0,
                                        );
                                    }
                                } else if natural_roll == 20 || modified_hit_roll > armor_class {
                                    // Target hit!
                                    let base_damage = rng.roll_dice(
                                        weapon_info.damage_n_dice,
                                        weapon_info.damage_die_type,
                                    );
                                    let attr_damage_bonus = attacker_attributes.might.bonus;
                                    let skill_damage_bonus =
                                        skill_bonus(Skill::Melee, &*attacker_skills);
                                    let weapon_damage_bonus = weapon_info.damage_bonus;

                                    let damage = i32::max(
                                        0,
                                        base_damage
                                            + attr_damage_bonus
                                            + skill_damage_bonus
                                            + weapon_damage_bonus,
                                    );
                                    SufferDamage::new_damage(&command_buffer, target, damage);
                                    log.entries.push(format!(
                                        "{} hits {}, for {} hp.",
                                        &attacker_name, &target_name, damage
                                    ));
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
                                } else {
                                    // Miss
                                    log.entries.push(format!(
                                        "{} attacks {}, but can't connect.",
                                        attacker_name, target_name
                                    ));
                                    if let Some(pos) = world.get_component::<Position>(target) {
                                        particle_builder.request(
                                            pos.x,
                                            pos.y,
                                            rltk::RGB::named(rltk::CYAN),
                                            rltk::RGB::named(rltk::BLACK),
                                            rltk::to_cp437('‼'),
                                            200.0,
                                        );
                                    }
                                }
                            } else {
                                console::log(&format!(
                                    "{}[{}] => {}[{}] - already dead - cannot do melee",
                                    attacker_name, entity, target_name, target
                                ));
                            }
                        }
                        _ => {
                            console::log(&format!(
                                "{} [{}] does not posses required components to fight",
                                target_name, target
                            ));
                        }
                    }
                }
            },
        )
}
