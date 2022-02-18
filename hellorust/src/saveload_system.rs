use specs::prelude::*;
use specs::saveload::{
    SimpleMarker,
    SimpleMarkerAllocator,
    SerializeComponents,
    DeserializeComponents,
    MarkedBuilder,
};
use std::convert::Infallible;
use std::fs;
use std::fs::File;
use std::path::Path;
use super::components::*;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
            SerializeComponents::<Infallible, SimpleMarker<SerializeMe>>::serialize(
                &( $ecs.read_storage::<$type>(), ),
                &$data.0,
                &$data.1,
                &mut $ser,
            )
            .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))] //?  Prevents web assembly trying to compile something it can't use
pub fn save_game(ecs : &mut World) {
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let savehelper = ecs.create_entity()
                        .with(SerializationHelper{ map : mapcopy })
                        .marked::<SimpleMarker<SerializeMe>>()
                        .build();

    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs, serializer, data,
            Position, Renderable, Player, Viewshed, Monster, ParticleLifetime,
            Name, BlocksTile, CombatStats, SufferDamage, DoesMelee,
            ThirstClock, MeleePowerBonus, DefenseBonus, AreaOfEffect, Confusion,
            Item, InBackpack, Consumable, Equippable, Equipped,
            Ranged, InflictsDamage, ProvidesHealing, ProvidesWater, MagicMapper,
            WantsToPickupItem, WantsToUseItem, WantsToDropItem, WantsToRemoveItem,
            SerializationHelper
        );
    }

    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}

#[cfg(target_arch = "wasm32")] //?  Protects from crashing when compiling to we assembly
pub fn save_game(_ecs : &mut World) { }

pub fn does_save_exist() -> bool { Path::new("./savegame.json").exists() }

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
            DeserializeComponents::<Infallible, _>::deserialize(
                &mut ( &mut $ecs.write_storage::<$type>(), ),
                &mut $data.0,
                &mut $data.1,
                &mut $data.2,
                &mut $de,
            )
            .unwrap();
        )*
    };
}

pub fn load_game(ecs: &mut World) {
    {
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() { to_delete.push(e); }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Clearing entities failed");
        }
    }

    let data = fs::read_to_string("./savegame.json").unwrap();

    let mut de = serde_json::Deserializer::from_str(&data);
    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>()
        );
        deserialize_individually!(
            ecs, de, d,
            Position, Renderable, Player, Viewshed, Monster, ParticleLifetime,
            Name, BlocksTile, CombatStats, SufferDamage, DoesMelee,
            ThirstClock, MeleePowerBonus, DefenseBonus, AreaOfEffect, Confusion,
            Item, InBackpack, Consumable, Equippable, Equipped,
            Ranged, InflictsDamage, ProvidesHealing, ProvidesWater, MagicMapper,
            WantsToPickupItem, WantsToUseItem, WantsToDropItem, WantsToRemoveItem,
            SerializationHelper
        );
    }

    let mut deleteme : Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e,h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            worldmap.tile_content = vec![Vec::new(); super::map::MAPCOUNT];
            deleteme = Some(e);
        }
        for (e,_p,pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs. delete_entity(deleteme.unwrap()).expect("Unable to delete deserializer")
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete save file");
    }
}
