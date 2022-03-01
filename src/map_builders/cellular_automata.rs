use rltk::RandomNumberGenerator;
use specs::prelude::*;
use super::{
    MapBuilder, Map, Rect, TileType, Position,
    apply_room_to_map, spawner,
    SHOW_MAPGEN_VISUALIZER,
};

const MIN_ROOM_SIZE : i32 = 8;

pub struct CellularAutomataBuilder {
    map : Map,
    starting_position : Position,
    depth : i32,
    history : Vec<Map>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&self) -> Map { self.map.clone() }
    fn get_starting_position(&self) -> Position { self.starting_position.clone() }
    fn get_snapshot_history(&self) -> Vec<Map> { self.history.clone() }

    fn build_map(&mut self) { self.build(); }

    fn spawn_entities(&mut self, ecs : &mut World) {  }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() { *v = true; }
        }
    }
}

impl CellularAutomataBuilder {
    pub fn new(new_depth : i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        for y in 1..self.map.height-2 {
            for x in 1..self.map.width-2 {
                let roll = rng.roll_dice(1,100);
                let idx = self.map.xy_idx(x,y);
                if roll > 55 { self.map.tiles[idx] = TileType::Floor; }
                else { self.map.tiles[idx] = TileType::Wall; }
            }
        }
        self.take_snapshot();
    }
}
