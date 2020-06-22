use serde::Deserialize;
use std::sync::Mutex;

mod prefab_master;
pub use prefab_master::*;
mod item_structs;
use item_structs::*;
mod mob_structs;
use mob_structs::*;
mod prop_structs;
use prop_structs::*;

#[derive(Deserialize, Debug)]
pub struct Prefabs {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
}

lazy_static! {
    pub static ref PREFABS: Mutex<PrefabMaster> = Mutex::new(PrefabMaster::empty());
}

rltk::embedded_resource!(PREFAB_FILE, "../../prefabs/spawns.ron");

pub fn load_prefabs() {
    rltk::link_resource!(PREFAB_FILE, "../../prefabs/spawns.ron");

    // Retrieve the raw data as an array of u8 (8-bit unsigned chars)
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../prefabs/spawns.ron".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(&raw_data).expect("Unable to convert to a valid UTF-8 string.");

    let decoder: Prefabs = ron::de::from_str(&raw_string).expect("Unable to parse RON");

    PREFABS.lock().unwrap().load(decoder);
}
