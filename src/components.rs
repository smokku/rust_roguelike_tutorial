use legion::prelude::*;
use rltk::{FontCharType, Point, RGB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToRemoveItem {
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
    Head,
    Torso,
    Legs,
    Feet,
    Hands,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "4239ace9-a158-4802-b797-591953c39ef3"]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq)]
#[uuid = "9aa18630-5131-45a7-a6b8-3878c4e25973"]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "667de082-404a-438d-8d8b-2c4f217a1017"]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "7242e30f-b971-4ad0-bcae-cc8ad67a5852"]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "0457c4c2-e26a-4d05-a4cb-d4322a41e876"]
pub struct ProvidesFood;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "74ea7770-fd58-43ce-a0b5-8ef6f8610d48"]
pub struct MagicMapper;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "8e5c82a0-f62a-46f8-95ea-b2531da310b1"]
pub struct Hidden;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "1f0408cb-f7f2-45aa-b2d6-0aac7bae3a7d"]
pub struct EntryTrigger;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "bfd72dfa-446e-49bc-99ab-259eb6e0cbf3"]
pub struct SingleActivation;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "4e87fd9c-ce64-4464-bf0f-ea1bca2d3827"]
pub struct BlocksVisibility;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "9998e1d1-bc60-4ebc-8edd-cf7b112df34b"]
pub struct Door {
    pub open: bool,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "8937d655-3173-4646-9ff2-de4cce96285f"]
pub struct Bystander;

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "401102d1-3cbb-451f-8989-c5b9aa7539bb"]
pub struct Vendor;

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "a15abace-8292-4203-88e8-c2ba0093e789"]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "614c79e7-c29f-4f46-9ed8-1dd2979ffc34"]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Melee,
    Defense,
    Magic,
}

#[derive(TypeUuid, Clone, Debug, Serialize, Deserialize)]
#[uuid = "28e5dc44-b610-4152-a3be-ce4e466f94a5"]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "f2f8a991-a90c-46e0-b986-25da16ff384e"]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub experience: i32,
    pub level: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Might,
    Quickness,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "43a4e3d5-dc45-465a-9c42-4672dfca6c16"]
pub struct MeleeWeapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(TypeUuid, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "64f8327b-24cc-409e-8567-aa73ac9923ce"]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: EquipmentSlot,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(TypeUuid, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[uuid = "3ad801a0-461e-4af3-81c5-3925eba81a6f"]
pub struct NaturalAttackDefense {
    pub armor_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>,
}
