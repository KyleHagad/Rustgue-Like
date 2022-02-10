use rltk::{ GameState, Rltk, RGB, RandomNumberGenerator, Point };
use specs::prelude::*;


mod components; // import components
pub use components::*; // make its public contents available
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

pub struct State {
    pub ecs: World,
    pub runstate: RunState
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {//  GameState is a trait implemented on State
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }


        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
        }
    }
}

fn main() -> rltk::BError {
    println!("Gathering Mana...");

    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rust Rouge Rogue")
        .build()?;
    // Create a game-state
    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running
    };
    // Register components to the game-state
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let map : Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    // Monster Spawner
    let mut rng = RandomNumberGenerator::new();
    for (i,room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();
        let glyph : rltk::FontCharType;
        let name : String;
        let roll = rng.roll_dice(1,2);

        match roll {
            1 => { glyph = rltk::to_cp437('O'); name = "Orc".to_string(); }
            _ => { glyph = rltk::to_cp437('G'); name = "Goblin".to_string(); }
            // 1 => { glyph = rltk::to_cp437('▲') }
            // _ => { glyph = rltk::to_cp437('▼') }
        }
        gs.ecs.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::MAGENTA),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Monster{})
            .with(Name{ name: format!("{} #{}", &name, i) })
            .build();
    }

    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('⌂'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed{visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Name{ name: "Player".to_string() })
        .build();

    rltk::main_loop(context, gs) //  Calls into the `rltk` namespace to activate `main_loop
}
