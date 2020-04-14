use legion::prelude::*;
use rltk::{FontCharType, Point, RGB};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "f4e159d6-63de-4a3c-a21b-63f8f2bd19c9"]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "f24fa790-5156-4d0b-bf36-10421caee6d9"]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "70d29e4c-8cd9-40a3-8329-b95124bc53f2"]
pub struct Player;

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "cd7ba29f-2434-4c7f-8d00-ac1370b2c287"]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "b2f34c7c-63fa-4e6d-96a2-bbbeca8bedac"]
pub struct Monster;

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "886ff52d-3052-467e-aa9a-2f628a463e86"]
pub struct Name {
    pub name: String,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "8b2e566c-2e72-48b0-954b-dffb83051683"]
pub struct BlocksTile;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "c810b39a-edcd-4435-9529-1c6fee305a8f"]
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

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "56a6359c-d947-438a-a13f-dfe65174cb6d"]
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

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "d9a3d242-2918-4241-9342-4d49a6e54f7c"]
pub struct Item;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "e878ef86-1af2-426f-abf5-49e810f7061e"]
pub struct Consumable;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq)]
#[uuid = "98a23186-8084-40fb-938e-f0fa6b983286"]
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
    pub target: Option<Point>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "fde630bf-14fc-46e6-8cd9-36a2cbb0734a"]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "84480f6e-697a-4cd6-8d34-3aa0c7116d05"]
pub struct Ranged {
    pub range: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "707b602d-12ed-4f49-8f1a-94adea423f71"]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "d55f9594-4993-43d6-b536-c4228189abf7"]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "aa447dea-909f-4a99-81fa-412ec6f5317c"]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "4239ace9-a158-4802-b797-591953c39ef3"]
pub struct Equippable {
    pub slot: EquipmentSlot,
}
