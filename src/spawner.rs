use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use::specs::saveload::{ MarkedBuilder, SimpleMarker };
use::std::collections::HashMap;
use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, Monster, Map,
    BlocksTile, Rect, map::MAPWIDTH, TileType,
    Item, Consumable, ProvidesHealing,
    Ranged, InflictsDamage, AreaOfEffect, Confusion, MagicMapper, Hidden,
    Equippable, EquipmentSlot, MeleePowerBonus, DefenseBonus,
    ThirstClock, ThirstState, ProvidesWater, EntryTrigger, TriggersOnce,
    SerializeMe, random_table::RandomTable,
};

/// Spawns player & returns its entity
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
    ecs.create_entity()
        .with(Position{ x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('⌂'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed{visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{ name: "Player".to_string() })
        .with(CombatStats{ max_hp: 30, hp: 30, defense: 2, power: 5 })
        .with(ThirstClock{ state: ThirstState::Quenched, duration: 20 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

const MAX_MONSTERS : i32 = 4;

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Bloody Heart", 7)
        .add("Blood Vial", 11)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Map Scroll", 300)
        .add("Dagger", 3)
        .add("Sword", map_depth -1)
        .add("Shield", 3)
        .add("Tower Shield", map_depth -1)
        .add("Spike Trap", 6)
        .add("Snap Trap", 6)
}

/// Fill a room
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room : &Rect, map_depth: i32) {
    let mut possible_targets : Vec<usize> = Vec::new();
    {
        let map = ecs.fetch::<Map>();
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 +1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(ecs, &possible_targets, map_depth);
}

pub fn spawn_region(ecs: &mut World, area: &[usize], map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }


        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, area.len() as i32) - 1) as usize};
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(&mut rng));
            areas.remove(array_index);
        }
    }

    for spawn in spawn_points.iter() {
        spawn_entity(ecs, &spawn);
    }
}

fn spawn_entity(ecs : &mut World, spawn : &(&usize, &String)) {
    let x = (*spawn.0 % MAPWIDTH) as i32;
    let y = (*spawn.0 / MAPWIDTH) as i32;

    match spawn.1.as_ref() {
        "Goblin" => goblin(ecs, x,y),
        "Orc" => orc(ecs, x,y),
        "Health Potion" => health_potion(ecs, x,y),
        "Blood Vial" => blood(ecs, x,y),
        "Fireball Scroll" => fireball_scroll(ecs, x,y),
        "Confusion Scroll" => confusion_scroll(ecs, x,y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x,y),
        "Map Scroll" => map_scroll(ecs, x,y),
        "Dagger" => dagger(ecs, x,y),
        "Sword" => sword(ecs, x,y),
        "Shield" => shield(ecs, x,y),
        "Tower Shield" => tower_shield(ecs, x,y),
        "Spike Trap" => spike_trap(ecs, x,y),
        "Snap Trap" => snap_trap(ecs, x,y),
        _ => { },
    }
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable {
            glyph: rltk::to_cp437('ì'),
            fg: RGB::named(rltk::DARKCYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Dagger".to_string() })
        .with(Item{ })
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn sword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable {
            glyph: rltk::to_cp437('ï'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Sword".to_string() })
        .with(Item{ })
        .with(Equippable{ slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus{ power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable {
            glyph: rltk::to_cp437('ù'),
            fg: RGB::named(rltk::DARKCYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Shield".to_string() })
        .with(Item{ })
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable {
            glyph: rltk::to_cp437('ü'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Tower Shield".to_string() })
        .with(Item{ })
        .with(Equippable{ slot: EquipmentSlot::Shield })
        .with(DefenseBonus{ defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Spawns health potions
fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437('♥'), // ¡
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Bloody Heart".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(ProvidesHealing{ heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn blood(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Blood Vial".to_string() })
        .with(Item{})
        .with(ProvidesWater{})
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn map_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN3),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Map Scroll".to_string() })
        .with(Item{})
        .with(MagicMapper{})
        .with(Consumable{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Magic Missile Scroll
pub fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Magic Missile Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

/// Fireball Scroll
/// - Creates a fireball scroll at a given location
///
/// args: ecs, x, y
fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name: "Fireball Scroll".to_string() })
        .with(Item{})
        .with(Consumable{})
        .with(Ranged{ range: 6 })
        .with(InflictsDamage{ damage: 20})
        .with(AreaOfEffect{ radius: 3})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y:i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Confusion Scroll".to_string()})
        .with(Item{ })
        .with(Consumable{ })
        .with(Ranged{ range: 6 })
        .with(Confusion{ turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn spike_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Spike Trap".to_string() })
        .with(Hidden{})
        .with(EntryTrigger{})
        .with(InflictsDamage{ damage: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn snap_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x,y })
        .with(Renderable{
            glyph: rltk::to_cp437('v'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name{ name : "Snap Trap".to_string() })
        .with(Hidden{})
        .with(EntryTrigger{})
        .with(InflictsDamage{ damage: 6 })
        .with(TriggersOnce{})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    let orc_stats = CombatStats{
        max_hp: 16,
        hp: 16,
        defense: 1,
        power: 4,
    };
    monster(ecs, x,y, rltk::to_cp437('O'), "Orc", orc_stats);
}
fn goblin(ecs: &mut World, x: i32, y: i32) {
    let goblin_stats = CombatStats{
        max_hp: 6,
        hp: 6,
        defense: 0,
        power: 2,
    };
    monster(ecs, x,y, rltk::to_cp437('G'), "Goblin", goblin_stats);
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, name : S, stats : CombatStats) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(stats)
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}
