use rltk::rex::XpFile;

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    pub fn new() -> Self {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/SmallDungeon_80x50.xp").unwrap(),
        }
    }
}
