rltk::embedded_resource!(PREFAB_FILE, "../../prefabs/spawns.ron");

pub fn load_prefabs() {
    rltk::link_resource!(PREFAB_FILE, "../../prefabs/spawns.ron");
}
