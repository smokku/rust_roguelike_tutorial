use super::Prefabs;
use std::collections::HashMap;

pub struct PrefabMaster {
    prefabs: Prefabs,
    item_index: HashMap<String, usize>,
}

impl PrefabMaster {
    pub fn empty() -> Self {
        PrefabMaster {
            prefabs: Prefabs { items: Vec::new() },
            item_index: HashMap::new(),
        }
    }

    pub fn load(&mut self, prefabs: Prefabs) {
        self.prefabs = prefabs;
        self.item_index = HashMap::new();
        for (i, item) in self.prefabs.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
    }
}
