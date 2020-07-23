use super::{components::*, Map};
use legion::{
    entity::EntityAllocator,
    prelude::*,
    storage::{
        ArchetypeDescription, ComponentMeta, ComponentResourceSet, ComponentTypeId, TagMeta,
        TagStorage, TagTypeId,
    },
};
use serde::{
    de::{self, DeserializeSeed, IgnoredAny, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    any::TypeId, cell::RefCell, collections::HashMap, fs, iter::FromIterator, marker::PhantomData,
    path::Path, ptr::NonNull,
};
use type_uuid::TypeUuid;

struct ComponentDeserializer<'de, T: Deserialize<'de>> {
    ptr: *mut T,
    _marker: PhantomData<&'de T>,
}

impl<'de, T: Deserialize<'de> + 'static> DeserializeSeed<'de> for ComponentDeserializer<'de, T> {
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <T as Deserialize<'de>>::deserialize(deserializer)?;
        unsafe {
            std::ptr::write(self.ptr, value);
        }
        Ok(())
    }
}

struct ComponentSeqDeserializer<'a, T> {
    get_next_storage_fn: &'a mut dyn FnMut() -> Option<(NonNull<u8>, usize)>,
    _marker: PhantomData<T>,
}

impl<'de, 'a, T: for<'b> Deserialize<'b> + 'static> DeserializeSeed<'de>
    for ComponentSeqDeserializer<'a, T>
{
    type Value = ();
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}
impl<'de, 'a, T: for<'b> Deserialize<'b> + 'static> Visitor<'de>
    for ComponentSeqDeserializer<'a, T>
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("sequence of objects")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let size = seq.size_hint();
        for _ in 0..size.unwrap_or(std::usize::MAX) {
            match (self.get_next_storage_fn)() {
                Some((storage_ptr, storage_len)) => {
                    let storage_ptr = storage_ptr.as_ptr() as *mut T;
                    for idx in 0..storage_len {
                        let element_ptr = unsafe { storage_ptr.add(idx) };

                        if seq
                            .next_element_seed(ComponentDeserializer {
                                ptr: element_ptr,
                                _marker: PhantomData,
                            })?
                            .is_none()
                        {
                            panic!(
                                "expected {} elements in chunk but only {} found",
                                storage_len, idx
                            );
                        }
                    }
                }
                None => {
                    if seq.next_element::<IgnoredAny>()?.is_some() {
                        panic!("unexpected element when there was no storage space available");
                    } else {
                        // No more elements and no more storage - that's what we want!
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
struct TagRegistration {
    uuid: type_uuid::Bytes,
    ty: TypeId,
    tag_serialize_fn: fn(&TagStorage, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    tag_deserialize_fn: fn(
        deserializer: &mut dyn erased_serde::Deserializer,
        &mut TagStorage,
    ) -> Result<(), erased_serde::Error>,
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
            tag_deserialize_fn: |deserializer, tag_storage| {
                // TODO implement visitor to avoid allocation of Vec
                let tag_vec = <Vec<T> as Deserialize>::deserialize(deserializer)?;
                for tag in tag_vec {
                    // Tag types should line up, making this safe
                    unsafe {
                        tag_storage.push(tag);
                    }
                }
                Ok(())
            },
            register_tag_fn: |desc| {
                desc.register_tag::<T>();
            },
        }
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
struct ComponentRegistration {
    uuid: type_uuid::Bytes,
    ty: TypeId,
    comp_serialize_fn: fn(&ComponentResourceSet, &mut dyn FnMut(&dyn erased_serde::Serialize)),
    comp_deserialize_fn: fn(
        deserializer: &mut dyn erased_serde::Deserializer,
        get_next_storage_fn: &mut dyn FnMut() -> Option<(NonNull<u8>, usize)>,
    ) -> Result<(), erased_serde::Error>,
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
            comp_deserialize_fn: |deserializer, get_next_storage_fn| {
                let comp_seq_deser = ComponentSeqDeserializer::<T> {
                    get_next_storage_fn,
                    _marker: PhantomData,
                };
                comp_seq_deser.deserialize(deserializer)?;
                Ok(())
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
        self.tag_types.get(&ty.type_id()).is_some()
    }
    fn can_serialize_component(&self, ty: &ComponentTypeId, _meta: &ComponentMeta) -> bool {
        self.comp_types.get(&ty.type_id()).is_some()
    }
    fn serialize_archetype_description<S: Serializer>(
        &self,
        serializer: S,
        archetype_desc: &ArchetypeDescription,
    ) -> Result<S::Ok, S::Error> {
        let tags_to_serialize = archetype_desc
            .tags()
            .iter()
            .filter_map(|(ty, _)| self.tag_types.get(&ty.type_id()))
            .map(|reg| reg.uuid)
            .collect::<Vec<_>>();
        let components_to_serialize = archetype_desc
            .components()
            .iter()
            .filter_map(|(ty, _)| self.comp_types.get(&ty.type_id()))
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
        if let Some(reg) = self.comp_types.get(&component_type.type_id()) {
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
        if let Some(reg) = self.tag_types.get(&tag_type.type_id()) {
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

struct DeserializeImpl {
    tag_types: HashMap<TypeId, TagRegistration>,
    comp_types: HashMap<TypeId, ComponentRegistration>,
    tag_types_by_uuid: HashMap<type_uuid::Bytes, TagRegistration>,
    comp_types_by_uuid: HashMap<type_uuid::Bytes, ComponentRegistration>,
    entity_map: RefCell<HashMap<uuid::Bytes, Entity>>,
}
impl legion::serialize::de::WorldDeserializer for DeserializeImpl {
    fn deserialize_archetype_description<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<ArchetypeDescription, <D as Deserializer<'de>>::Error> {
        let serialized_desc =
            <SerializedArchetypeDescription as Deserialize>::deserialize(deserializer)?;
        let mut desc = ArchetypeDescription::default();
        for tag in serialized_desc.tag_types {
            if let Some(reg) = self.tag_types_by_uuid.get(&tag) {
                (reg.register_tag_fn)(&mut desc);
            }
        }
        for comp in serialized_desc.component_types {
            if let Some(reg) = self.comp_types_by_uuid.get(&comp) {
                (reg.register_comp_fn)(&mut desc);
            }
        }
        Ok(desc)
    }
    fn deserialize_components<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
        component_type: &ComponentTypeId,
        _component_meta: &ComponentMeta,
        get_next_storage_fn: &mut dyn FnMut() -> Option<(NonNull<u8>, usize)>,
    ) -> Result<(), <D as Deserializer<'de>>::Error> {
        if let Some(reg) = self.comp_types.get(&component_type.type_id()) {
            let mut erased = erased_serde::Deserializer::erase(deserializer);
            (reg.comp_deserialize_fn)(&mut erased, get_next_storage_fn)
                .map_err(<<D as serde::Deserializer<'de>>::Error as serde::de::Error>::custom)?;
        } else {
            <IgnoredAny>::deserialize(deserializer)?;
        }
        Ok(())
    }
    fn deserialize_tags<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
        tag_type: &TagTypeId,
        _tag_meta: &TagMeta,
        tags: &mut TagStorage,
    ) -> Result<(), <D as Deserializer<'de>>::Error> {
        if let Some(reg) = self.tag_types.get(&tag_type.0) {
            let mut erased = erased_serde::Deserializer::erase(deserializer);
            (reg.tag_deserialize_fn)(&mut erased, tags)
                .map_err(<<D as serde::Deserializer<'de>>::Error as serde::de::Error>::custom)?;
        } else {
            <IgnoredAny>::deserialize(deserializer)?;
        }
        Ok(())
    }
    fn deserialize_entities<'de, D: Deserializer<'de>>(
        &self,
        deserializer: D,
        entity_allocator: &EntityAllocator,
        entities: &mut Vec<Entity>,
    ) -> Result<(), <D as Deserializer<'de>>::Error> {
        let entity_uuids = <Vec<uuid::Bytes> as Deserialize>::deserialize(deserializer)?;
        let mut entity_map = self.entity_map.borrow_mut();
        for id in entity_uuids {
            let entity = entity_allocator.create_entity();
            entity_map.insert(id, entity);
            entities.push(entity);
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
fn get_serializer() -> SerializeImpl {
    let comp_registrations = [
        ComponentRegistration::of::<Map>(),
        ComponentRegistration::of::<Position>(),
        ComponentRegistration::of::<Renderable>(),
        ComponentRegistration::of::<Viewshed>(),
        ComponentRegistration::of::<Name>(),
        ComponentRegistration::of::<SufferDamage>(),
        ComponentRegistration::of::<Ranged>(),
        ComponentRegistration::of::<InflictsDamage>(),
        ComponentRegistration::of::<AreaOfEffect>(),
        ComponentRegistration::of::<Confusion>(),
        ComponentRegistration::of::<ProvidesHealing>(),
        // ComponentRegistration::of::<InBackpack>(), // FIXME: Do not loose player backpack!
        ComponentRegistration::of::<Equippable>(),
        // ComponentRegistration::of::<Equipped>(), // FIXME: Do not loose player equipment!
        ComponentRegistration::of::<DefenseBonus>(),
        ComponentRegistration::of::<ParticleLifetime>(),
        ComponentRegistration::of::<HungerClock>(),
        ComponentRegistration::of::<Door>(),
        ComponentRegistration::of::<Quips>(),
        ComponentRegistration::of::<Attributes>(),
        ComponentRegistration::of::<Skills>(),
        ComponentRegistration::of::<Pools>(),
        ComponentRegistration::of::<MeleeWeapon>(),
    ];
    let tag_registrations = [
        TagRegistration::of::<Player>(),
        TagRegistration::of::<Monster>(),
        TagRegistration::of::<BlocksTile>(),
        TagRegistration::of::<Item>(),
        TagRegistration::of::<Consumable>(),
        TagRegistration::of::<ProvidesFood>(),
        TagRegistration::of::<MagicMapper>(),
        TagRegistration::of::<Hidden>(),
        TagRegistration::of::<EntryTrigger>(),
        TagRegistration::of::<SingleActivation>(),
        TagRegistration::of::<BlocksVisibility>(),
        TagRegistration::of::<Bystander>(),
        TagRegistration::of::<Vendor>(),
    ];

    SerializeImpl {
        comp_types: HashMap::from_iter(comp_registrations.iter().map(|reg| (reg.ty, reg.clone()))),
        tag_types: HashMap::from_iter(tag_registrations.iter().map(|reg| (reg.ty, reg.clone()))),
        entity_map: RefCell::new(HashMap::new()),
    }
}

fn get_deserializer() -> DeserializeImpl {
    let ser_helper = get_serializer();
    DeserializeImpl {
        tag_types_by_uuid: HashMap::from_iter(
            ser_helper
                .tag_types
                .iter()
                .map(|reg| (reg.1.uuid, reg.1.clone())),
        ),
        comp_types_by_uuid: HashMap::from_iter(
            ser_helper
                .comp_types
                .iter()
                .map(|reg| (reg.1.uuid, reg.1.clone())),
        ),
        tag_types: ser_helper.tag_types,
        comp_types: ser_helper.comp_types,
        // re-use the entity-uuid mapping
        entity_map: RefCell::new(HashMap::from_iter(
            ser_helper
                .entity_map
                .into_inner()
                .into_iter()
                .map(|(e, uuid)| (uuid, e)),
        )),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(world: &mut World, map: &Map) {
    // Add Map as an entity
    let map_entity = world.insert((), vec![(map.clone(),)])[0];

    let ser_helper = get_serializer();
    let writer = std::fs::File::create("./savegame.json").unwrap();
    let serializable = legion::serialize::ser::serializable_world(&world, &ser_helper);
    serde_json::to_writer_pretty(writer, &serializable).expect("Unable to save game");

    // Clean up
    world.delete(map_entity);
}

#[cfg(target_arch = "wasm32")]
pub fn save_game(_world: &mut World, _map: &Map) {}

pub fn does_save_exist() -> bool {
    Path::new("./savegame.json").exists()
}

pub fn load_game(mut world: &mut World) {
    world.delete_all();

    let de_helper = get_deserializer();
    let data = fs::read_to_string("./savegame.json").unwrap();
    let mut deserializer = serde_json::Deserializer::from_str(&data);
    legion::serialize::de::deserialize(&mut world, &de_helper, &mut deserializer).unwrap();
}

pub fn delete_save() {
    let _ = fs::remove_file("./savegame.json");
}
