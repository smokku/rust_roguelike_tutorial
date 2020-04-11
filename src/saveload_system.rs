use super::{components::*, Map};
use legion::{
    prelude::*,
    storage::{
        ArchetypeDescription, ComponentMeta, ComponentResourceSet, ComponentTypeId, TagMeta,
        TagStorage, TagTypeId,
    },
};
use serde::{Deserialize, Serialize, Serializer};
use std::fs::File;
use std::{any::TypeId, cell::RefCell, collections::HashMap};
use type_uuid::TypeUuid;

#[derive(Clone)]
struct TagRegistration {
    uuid: type_uuid::Bytes,
    ty: TypeId,
    tag_serialize_fn: fn(&TagStorage, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    register_tag_fn: fn(&mut ArchetypeDescription),
}

impl TagRegistration {
    fn of<
        T: TypeUuid
            + Serialize
            + for<'de> Deserialize<'de>
            + PartialEq
            + Clone
            + Send
            + Sync
            + 'static,
    >() -> Self {
        Self {
            uuid: T::UUID,
            ty: TypeId::of::<T>(),
            tag_serialize_fn: |tag_storage, serialize_fn| {
                // it's safe because we know this is the correct type due to lookup
                let slice = unsafe { tag_storage.data_slice::<T>() };
                serialize_fn(&&*slice);
            },
            register_tag_fn: |desc| {
                desc.register_tag::<T>();
            },
        }
    }
}

#[derive(Clone)]
struct ComponentRegistration {
    uuid: type_uuid::Bytes,
    ty: TypeId,
    comp_serialize_fn: fn(&ComponentResourceSet, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    register_comp_fn: fn(&mut ArchetypeDescription),
}

impl ComponentRegistration {
    fn of<T: TypeUuid + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static>() -> Self {
        Self {
            uuid: T::UUID,
            ty: TypeId::of::<T>(),
            comp_serialize_fn: |comp_storage, serialize_fn| {
                // it's safe because we know this is the correct type due to lookup
                let slice = unsafe { comp_storage.data_slice::<T>() };
                serialize_fn(&*slice);
            },
            register_comp_fn: |desc| {
                desc.register_component::<T>();
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedArchetypeDescription {
    tag_types: Vec<type_uuid::Bytes>,
    component_types: Vec<type_uuid::Bytes>,
}

struct SerializeImpl {
    tag_types: HashMap<TypeId, TagRegistration>,
    comp_types: HashMap<TypeId, ComponentRegistration>,
    entity_map: RefCell<HashMap<Entity, uuid::Bytes>>,
}
impl legion::serialize::ser::WorldSerializer for SerializeImpl {
    fn can_serialize_tag(&self, ty: &TagTypeId, _meta: &TagMeta) -> bool {
        self.tag_types.get(&ty.0).is_some()
    }
    fn can_serialize_component(&self, ty: &ComponentTypeId, _meta: &ComponentMeta) -> bool {
        self.comp_types.get(&ty.0).is_some()
    }
    fn serialize_archetype_description<S: Serializer>(
        &self,
        serializer: S,
        archetype_desc: &ArchetypeDescription,
    ) -> Result<S::Ok, S::Error> {
        let tags_to_serialize = archetype_desc
            .tags()
            .iter()
            .filter_map(|(ty, _)| self.tag_types.get(&ty.0))
            .map(|reg| reg.uuid)
            .collect::<Vec<_>>();
        let components_to_serialize = archetype_desc
            .components()
            .iter()
            .filter_map(|(ty, _)| self.comp_types.get(&ty.0))
            .map(|reg| reg.uuid)
            .collect::<Vec<_>>();
        SerializedArchetypeDescription {
            tag_types: tags_to_serialize,
            component_types: components_to_serialize,
        }
        .serialize(serializer)
    }
    fn serialize_components<S: Serializer>(
        &self,
        serializer: S,
        component_type: &ComponentTypeId,
        _component_meta: &ComponentMeta,
        components: &ComponentResourceSet,
    ) -> Result<S::Ok, S::Error> {
        if let Some(reg) = self.comp_types.get(&component_type.0) {
            let result = RefCell::new(None);
            let serializer = RefCell::new(Some(serializer));
            {
                let mut result_ref = result.borrow_mut();
                (reg.comp_serialize_fn)(components, &mut |serialize| {
                    result_ref.replace(erased_serde::serialize(
                        serialize,
                        serializer.borrow_mut().take().unwrap(),
                    ));
                });
            }
            return result.borrow_mut().take().unwrap();
        }
        panic!(
            "received unserializable type {:?}, this should be filtered by can_serialize",
            component_type
        );
    }
    fn serialize_tags<S: Serializer>(
        &self,
        serializer: S,
        tag_type: &TagTypeId,
        _tag_meta: &TagMeta,
        tags: &TagStorage,
    ) -> Result<S::Ok, S::Error> {
        if let Some(reg) = self.tag_types.get(&tag_type.0) {
            let result = RefCell::new(None);
            let serializer = RefCell::new(Some(serializer));
            {
                let mut result_ref = result.borrow_mut();
                (reg.tag_serialize_fn)(tags, &mut |serialize| {
                    result_ref.replace(erased_serde::serialize(
                        serialize,
                        serializer.borrow_mut().take().unwrap(),
                    ));
                });
            }
            return result.borrow_mut().take().unwrap();
        }
        panic!(
            "received unserializable type {:?}, this should be filtered by can_serialize",
            tag_type
        );
    }
    fn serialize_entities<S: Serializer>(
        &self,
        serializer: S,
        entities: &[Entity],
    ) -> Result<S::Ok, S::Error> {
        let mut uuid_map = self.entity_map.borrow_mut();
        serializer.collect_seq(entities.iter().map(|e| {
            *uuid_map
                .entry(*e)
                .or_insert_with(|| *uuid::Uuid::new_v4().as_bytes())
        }))
    }
}

pub fn save_game<'a>(world: &mut World, map: &'a Map) {
    let comp_registrations = [
        ComponentRegistration::of::<Map>(),
        ComponentRegistration::of::<Position>(),
        ComponentRegistration::of::<Renderable>(),
        ComponentRegistration::of::<Viewshed>(),
        ComponentRegistration::of::<Name>(),
        ComponentRegistration::of::<CombatStats>(),
        ComponentRegistration::of::<SufferDamage>(),
        ComponentRegistration::of::<Ranged>(),
        ComponentRegistration::of::<InflictsDamage>(),
        ComponentRegistration::of::<AreaOfEffect>(),
        ComponentRegistration::of::<Confusion>(),
        ComponentRegistration::of::<ProvidesHealing>(),
        // ComponentRegistration::of::<InBackpack>(), // FIXME: Do not loose player backpack!
    ];
    let tag_registrations = [
        TagRegistration::of::<Player>(),
        TagRegistration::of::<Monster>(),
        TagRegistration::of::<BlocksTile>(),
        TagRegistration::of::<Item>(),
        TagRegistration::of::<Consumable>(),
    ];

    use std::iter::FromIterator;
    let ser_helper = SerializeImpl {
        comp_types: HashMap::from_iter(comp_registrations.iter().map(|reg| (reg.ty, reg.clone()))),
        tag_types: HashMap::from_iter(tag_registrations.iter().map(|reg| (reg.ty, reg.clone()))),
        entity_map: RefCell::new(HashMap::new()),
    };

    // Add Map as an entity
    let map_entity = world.insert((), vec![(map.clone(),)])[0];

    let writer = File::create("./savegame.json").unwrap();
    let serializable = legion::serialize::ser::serializable_world(&world, &ser_helper);
    serde_json::to_writer_pretty(writer, &serializable).expect("Unable to save game");

    // Clean up
    world.delete(map_entity);
}
