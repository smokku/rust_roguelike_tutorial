use super::{gamelog::GameLog, Map, Name, Player, Pools, Position, RunState, SufferDamage};
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
    let query = Read::<Pools>::query();
    for (victim, stats) in query.iter_entities(world) {
        if stats.hit_points.current < 1 {
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
