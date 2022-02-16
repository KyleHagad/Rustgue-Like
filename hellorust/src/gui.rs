use rltk::{ RGB, Rltk, Point, VirtualKeyCode };
use specs::prelude::*;
use super::{ RunState, Map, CombatStats, Player, GameLog, Name, Position, State, InBackpack, Viewshed, Equipped };

pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    let (
        prp, blk, ylw, crm, mvr, gld
    ) = (
        RGB::named(rltk::PURPLE), RGB::named(rltk::BLACK), RGB::named(rltk::YELLOW), RGB::named(rltk::CRIMSON), RGB::named(rltk::MEDIUMVIOLETRED), RGB::named(rltk::GOLD)
    );

    ctx.draw_box(0, 43, 79, 6, prp, blk);

    //?  Health display
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, ylw, blk, &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, crm, blk);

        let log = ecs.fetch::<GameLog>();
        let mut y = 44;
        for s in log.entries.iter().rev() {
            if y < 49 { ctx.print(2, y, s); }
            y += 1;
        }
    }

    //?  Depth Display
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {}", map.depth);
    ctx.print_color(2, 43, gld, blk, &depth);

    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, mvr);
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

pub fn drop_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter( |item| item.0.owner == *player_entity);
    let count = inventory.count();

    let (pnk, blk, ylw) = (RGB::named(rltk::LIGHTPINK), RGB::named(rltk::BLACK), RGB::named(rltk::KHAKI));

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, pnk, blk);
    ctx.print_color(18, y-2, ylw, blk, "Trash Item?");
    ctx.print_color(18, y+count as i32 +1, ylw, blk, "ESC closes backpack");

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

pub fn remove_item_menu(gs : &mut State, ctx : &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<Equipped>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names).join().filter( |item| item.0.owner == *player_entity );
    let count = inventory.count();

    let (ylw, pnk, blk) = (RGB::named(rltk::KHAKI), RGB::named(rltk::LIGHTPINK), RGB::named(rltk::BLACK));

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(15, y-2, 31, (count+3) as i32, pnk, blk);
    ctx.print_color(18, y-2, ylw, blk, "Unequip which item?");
    ctx.print_color(18, y+count as i32 +1, ylw, blk, "ESC Stop looking");

    let mut equippable : Vec<Entity> = Vec::new();
    let mut j = 0;
    for (entity, _pack, name) in (&entities, &backpack, &names).join().filter( |item| item.1.owner == *player_entity ) {
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

pub fn ranged_target(gs: &mut State, ctx : &mut Rltk, range : i32) -> (ItemMenuResult, Option<Point>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();
    let (ylw, blk, blu, cyn, red) = (
        RGB::named(rltk::KHAKI), RGB::named(rltk::BLACK), RGB::named(rltk::BLUE), RGB::named(rltk::CYAN), RGB::named(rltk::CRIMSON)
    );

    ctx.print_color(5,0, ylw, blk, "Select Target:");

    let mut available_cells = Vec::new();
    let visible = viewsheds.get(*player_entity);
    if let Some(visible) = visible {
        for idx in visible.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *idx);
            if distance <= range as f32 {
                ctx.set_bg(idx.x, idx.y, blu);
                available_cells.push(idx);
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    let mouse_pos = ctx.mouse_pos();
    let mut valid_target = false;
    for idx in available_cells.iter() { if idx.x == mouse_pos.0 && idx.y == mouse_pos.1 { valid_target = true; } }
    if valid_target {
        ctx.set_bg(mouse_pos.0,mouse_pos.1, cyn);
        if ctx.left_click {
            return (ItemMenuResult::Selected, Some(Point::new(mouse_pos.0,mouse_pos.1)));
        }
    } else {
        ctx.set_bg(mouse_pos.0,mouse_pos.1, red);
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit
}
#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection{ selected : MainMenuSelection},
    Selected{ selected : MainMenuSelection }
}

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();
    let (ylw, blk, mga, pnk) = (RGB::named(rltk::KHAKI), RGB::named(rltk::BLACK), RGB::named(rltk::MAGENTA), RGB::named(rltk::LIGHTPINK));

    ctx.print_color_centered(15, ylw, blk, "Rouge Rust Rogue");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == NewGame {
            ctx.print_color_centered(21, mga, blk, "Start Hunting");
        } else {
            ctx.print_color_centered(21, pnk, blk, "Start Hunting");
        }

        if save_exists {
            if selection == LoadGame {
                ctx.print_color_centered(23, mga, blk, "Continue Hunt");
            } else {
                ctx.print_color_centered(23, pnk, blk, "Continue Hunt");
            }
        }

        if selection == Quit {
            ctx.print_color_centered(25, mga, blk, "Quit");
        } else {
            ctx.print_color_centered(25, pnk, blk, "Quit");
        }

        use MainMenuSelection::*;
        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: Quit } }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            NewGame => newselection = LoadGame,
                            LoadGame => newselection = Quit,
                            Quit => newselection = NewGame
                        }
                        if newselection == LoadGame && !save_exists { newselection = Quit; }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            NewGame => newselection = Quit,
                            LoadGame => newselection = NewGame,
                            Quit => newselection = LoadGame
                        }
                        if newselection == LoadGame && !save_exists { newselection = NewGame; }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected: selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}
