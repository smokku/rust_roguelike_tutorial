use legion::prelude::*;
use rltk::RGB;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Renderable {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player;

#[derive(Clone, Debug, PartialEq)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monster;

#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlocksTile;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(command_buffer: &CommandBuffer, victim: Entity, amount: i32) {
        command_buffer.exec_mut(move |world| {
            let mut dmg = if let Some(suffering) = world.get_component::<SufferDamage>(victim) {
                (*suffering).clone()
            } else {
                SufferDamage { amount: Vec::new() }
            };

            dmg.amount.push(amount);
            world
                .add_component(victim, dmg)
                .expect("Unable to insert damage");
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Consumable;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToUseItem {
    pub item: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InflictsDamage {
    pub damage: i32,
}
