use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use super::{ CombatStats, Player, GameLog, Map, Name, Position, State, InBackpack };

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::PURPLE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::CRIMSON), RGB::named(rltk::BLACK));

        let log = ecs.fetch::<GameLog>();
        let mut y = 44;
        for s in log.entries.iter().rev() {
            if y < 49 { ctx.print(2, y, s); }
            y += 1;
        }
    }

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MEDIUMVIOLETRED));
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height { return; }
    let mut tooltip : Vec<String> = Vec::new();
    for (name, position) in (&names, &positions).join() {
        let idx = map.xy_idx(position.x, position.y);
        if position.x == mouse_pos.0 && position.y == mouse_pos.1 && map.visible_tiles[idx] {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width: i32 = 0;
        for s in tooltip.iter() {
            if width < s.len() as i32 { width = s.len() as i32; }
        }
        width += 3;
        let (tt_fg, tt_bg) = (RGB::named(rltk::LIGHTPINK), RGB::named(rltk::DARKSLATEGREY));
        if mouse_pos.0 > 40 {
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, tt_fg, tt_bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i, y, tt_fg, tt_bg, &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, tt_fg, tt_bg, &"->".to_string());
        } else {
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, tt_fg, tt_bg, s);
                let padding = (width - s.len() as i32) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i, y, tt_fg, tt_bg, &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, tt_fg, tt_bg, &"<-".to_string());
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ItemMenuResult { Cancel, NoResponse, Selected }
pub fn show_inventory(gs: &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter( |item| item.0.owner == *player_entity );
    let count = inventory.count();

    let (pnk, blk, ylw) = (RGB::named(rltk::LIGHTPINK), RGB::named(rltk::BLACK), RGB::named(rltk::KHAKI));

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, pnk, blk);
    ctx.print_color(18, y-2, ylw, blk, "Backpack");
    ctx.print_color(18, y+count as i32+1, ylw, blk, "ESC to close");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter( |item| item.1.owner == *player_entity) {
        ctx.set(17, y, pnk, blk, rltk::to_cp437('('));
        ctx.set(18, y, ylw, blk, 97+j as rltk::FontCharType);
        ctx.set(19, y, pnk, blk, rltk::to_cp437(')'));

        ctx.print(21, y, &name.name.to_string());
        equippable.push(entity);
        y += 1;
        j += 1;
    }

    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => {
            match key {
                VirtualKeyCode::Escape => { (ItemMenuResult::Cancel, None) }
                _ => {
                    let selection = rltk::letter_to_option(key);
                    if selection > -1 && selection < count as i32 {
                        return (ItemMenuResult::Selected, Some(equippable[selection as usize]));
                    }
                    (ItemMenuResult::NoResponse, None)
                }
            }
        }
    }
}
