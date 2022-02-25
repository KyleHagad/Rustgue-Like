use rltk::{ rex::XpFile };

rltk::embedded_resource!(SMALL_DUNGEON, "../resources/cubecircles.xp");

pub struct RexAssets { pub menu : XpFile }
impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/cubecircles.xp");

        RexAssets {
            menu : XpFile::from_resource("../resources/cubecircles.xp").unwrap()
        }
    }
}
