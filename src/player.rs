use rltk::{ Rltk, Point };
use specs::prelude::*;
use std::cmp::{ max, min };
use super::{
    Map, TileType, Position, State, RunState, GameLog, Player, Monster,
    Viewshed, CombatStats, DoesMelee, Item, WantsToPickupItem, EntityMoved,
    ThirstClock, ThirstState,
};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut does_melee = ecs.write_storage::<DoesMelee>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();

    for (entity, _player, pos, viewshed) in (&entities, &players, &mut positions, &mut viewsheds).join() {
        if pos.x + delta_x < 1 || pos.x + delta_x > map.width-1 || pos.y + delta_y < 1 || pos.y + delta_y > map.height - 1 { return; }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                does_melee.insert(entity, DoesMelee{ target: *potential_target }).expect("Add target failed");
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert moved marker.");

            viewshed.dirty = true;

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert picks up item");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let mut unseen = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => { }
                Some(_) => { can_heal = false; unseen = false;}
            }
        }
    }

    let mut log = ecs.fetch_mut::<GameLog>();

    let thirst_clocks = ecs.read_storage::<ThirstClock>();
    let tc = thirst_clocks.get(*player_entity);
    if let Some(tc) = tc {
        match tc.state {
            ThirstState::Thirsty => can_heal = false,
            ThirstState::Parched => can_heal = false,
            _ => { }
        }
    }

    if can_heal {
        let mut health_components = ecs.write_storage::<CombatStats>();
        let player_hp = health_components.get_mut(*player_entity).unwrap();
        player_hp.hp = i32::min(player_hp.hp + 1, player_hp.max_hp);
        if player_hp.hp >= player_hp.max_hp {
            log.entries.push("You wait".to_string());
        } else {
            log.entries.push("You rest a moment to catch your breath.".to_string());
        }
    } else if unseen {
        log.entries.push("Your thirst prevents rest.".to_string());
    } else {
        log.entries.push("You wait. There is an enemy nearby.".to_string());
    }

    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use rltk::VirtualKeyCode::*;
    match ctx.key {
        None => { return RunState::AwaitingInput }
        Some(key) => match key {
            Left | A => try_move_player(-1, 0, &mut gs.ecs),

            Right | F => try_move_player(1, 0, &mut gs.ecs),

            Up | D => try_move_player(0, -1, &mut gs.ecs),

            Down | S => try_move_player(0, 1, &mut gs.ecs),

            E => try_move_player(1, -1, &mut gs.ecs), //? NE

            W => try_move_player(-1, -1, &mut gs.ecs), //? NW

            C => try_move_player(1, 1, &mut gs.ecs), //? SE

            X => try_move_player(-1, 1, &mut gs.ecs), //? SW

            G => get_item(&mut gs.ecs),

            Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            I => return RunState::ShowInventory,

            T => return RunState::ShowDropItem,

            R => return RunState::ShowRemoveItem,

            Escape => return RunState::SaveGame,

            Space => return skip_turn(&mut gs.ecs),

            _ => { return RunState::AwaitingInput }
        },
    }

    RunState::PlayerTurn
}
