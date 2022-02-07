use rltk::{Rltk, GameState};

struct State {} //  No type definition, means this is a `void` function
 //  GameState is a trait implemented on State
impl GameState for State {
  //  Tick is a method on the trait GameState
  fn tick(&mut self, ctx : &mut Rltk) { // '&' is a reference to the original; 'mut' is a mutable reference
    ctx.cls();
    ctx.print(1, 1, "Hello, RLTK!");
  }
}

fn main() -> rltk::BError {
  println!("Initializing...");

  use rltk::RltkBuilder;
  let context = RltkBuilder::simple80x50()
    .with_title("Rust Rouge Rogue")
    .build()?;
  let gs = State{ }; //  Assigns a copy of State to `gs`
  rltk::main_loop(context, gs) //  Calls into the `rltk` namespace to activate `main_loop
}
