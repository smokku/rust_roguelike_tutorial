use super::{gamelog::GameLog, HungerClock, HungerState, RunState, SufferDamage};
use legion::prelude::*;

pub fn build() -> Box<(dyn Schedulable + 'static)> {
    SystemBuilder::new("hunger")
        .read_resource::<Entity>() // The Player
        .read_resource::<RunState>()
        .write_resource::<GameLog>()
        .with_query(Write::<HungerClock>::query())
        .build(
            |command_buffer, world, (player_entity, runstate, log), query| {
                for (entity, mut clock) in query.iter_entities_mut(world) {
                    let is_player = entity == **player_entity;

                    let proceed = match **runstate {
                        RunState::PlayerTurn => is_player,
                        RunState::MonsterTurn => !is_player,
                        _ => false,
                    };

                    if proceed {
                        clock.duration -= 1;
                        if clock.duration < 1 {
                            match clock.state {
                                HungerState::WellFed => {
                                    clock.state = HungerState::Normal;
                                    clock.duration = 200;
                                    if is_player {
                                        log.entries.push("You are no longer well fed.".to_string());
                                    }
                                }
                                HungerState::Normal => {
                                    clock.state = HungerState::Hungry;
                                    clock.duration = 200;
                                    if is_player {
                                        log.entries.push("You are hungry.".to_string());
                                    }
                                }
                                HungerState::Hungry => {
                                    clock.state = HungerState::Starving;
                                    clock.duration = 200;
                                    if is_player {
                                        log.entries.push("You are starving.".to_string());
                                    }
                                }
                                HungerState::Starving => {
                                    // Inflict damage from hunger
                                    if is_player {
                                        log.entries.push(
                                        "Your hunger is getting painful!. You suffer 1 hp damage."
                                            .to_string(),
                                    );
                                    }
                                    SufferDamage::new_damage(command_buffer, entity, 1);
                                }
                            }
                        }
                    }
                }
            },
        )
}
