use specs::prelude::*;
use specs::saveload::{
    SimpleMarker,
    SimpleMarkerAllocator,
    SerializeComponents,
    DeserializeComponents,
    MarkedBuilder,
};
use super::components::*;
use std::convert::Infallible;
use std::fs::File;

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
            Position, Renderable, Player, Viewshed, Monster,
            Name, BlocksTile, CombatStats, SufferDamage, DoesMelee,
            Item, Consumable, Ranged, InflictsDamage, AreaOfEffect, Confusion, ProvidesHealing,
            InBackpack, WantsToPickupItem, WantsToUseItem, WantsToDropItem,
            SerializationHelper
        );
    }

    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}