use rltk::{ RGB, RandomNumberGenerator };
use specs::prelude::*;
use::specs::saveload::{ MarkedBuilder, SimpleMarker };
use super::{
    CombatStats, Player, Renderable, Name, Position, Viewshed, Monster,
    BlocksTile, Rect, map::MAPWIDTH, Item, Consumable, ProvidesHealing,
    Ranged, InflictsDamage, AreaOfEffect, Confusion, SerializeMe
};

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;

/// Fill a room
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points : Vec<usize> = Vec::new();
    let mut item_spawn_points : Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS +2) - 3;


        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    for idx in monster_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_monster(ecs, x as i32, y as i32);
    }

    for idx in item_spawn_points.iter() {
        let x = *idx % MAPWIDTH;
        let y = *idx / MAPWIDTH;
        random_item(ecs, x as i32, y as i32);
    }
}

/// Random Item
fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll : i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1,6);
    }
    match roll {
        1 => { health_potion(ecs, x,y) }
        2 => { magic_missile_scroll(ecs, x,y) }
        3 => { fireball_scroll(ecs, x,y) }
        _ => { confusion_scroll(ecs, x,y) }
    }
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
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Spawns random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll : i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1,2);
    }
    match roll {
        1 => { orc(ecs, x,y) }
        _ => { goblin(ecs, x,y) }
    }
}

fn orc(ecs: &mut World, x: i32, y: i32) { monster(ecs, x,y, rltk::to_cp437('O'), "Orc"); }
fn goblin(ecs: &mut World, x: i32, y: i32) { monster(ecs, x,y, rltk::to_cp437('G'), "Goblin"); }

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, glyph : rltk::FontCharType, name : S) -> Entity {
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
        .with(CombatStats{ max_hp: 16, hp: 16, defense: 1, power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}
