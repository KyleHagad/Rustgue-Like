use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;
use specs_derive::Component;

// import components
mod components;
// make its public contents available
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;

#[derive(Component)]
struct LeftMover {}

pub struct State {
    ecs: World,
}

//  GameState is a trait implemented on State ⚵
impl GameState for State {
    //  Tick is a method on the trait GameState
    fn tick(&mut self, ctx: &mut Rltk) {
        // '&' is a reference to the original; 'mut' is a mutable reference
        ctx.cls();

        self.run_systems();

        player_input(self, ctx);

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    println!("Gathering Mana...");

    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rust Rouge Rogue")
        .build()?;

    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    let (rooms, map) = new_map_rooms_and_corridors();
    gs.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('⌂'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs) //  Calls into the `rltk` namespace to activate `main_loop
}
