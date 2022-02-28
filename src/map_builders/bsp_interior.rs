use super::{
    Map, MapBuilder, Rect, TileType, Position,
    spawner, apply_room_to_map,
    SHOW_MAPGEN_VISUALIZER, MAPWIDTH, MAPHEIGHT,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

const MIN_ROOM_SIZE : i32 = 8;

pub struct BspInteriorBuilder {
    map : Map,
    starting_position : Position,
    depth : i32,
    rooms : Vec<Rect>,
    history : Vec<Map>,
    rects : Vec<Rect>,
}

impl MapBuilder for BspInteriorBuilder {
    fn get_map(&self) -> Map { self.map.clone() }
    fn get_starting_position(&self) -> Position { self.starting_position.clone() }
    fn get_snapshot_history(&self) -> Vec<Map> { self.history.clone() }

    fn build_map(&mut self) { self.build(); }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() { *v = true; }
            self.history.push(snapshot);
        }
    }
}

impl BspInteriorBuilder {
    pub fn new(new_depth : i32) -> BspInteriorBuilder {
        BspInteriorBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y: 0 },
            depth : new_depth,
            rooms : Vec::new(),
            history : Vec::new(),
            rects : Vec::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.rects.clear();
        self.rects.push( Rect::new(1,1, self.map.width-2, self.map.height-2) );
        let first_room = self.rects[0];
        self.add_subrects(first_room, &mut rng);

        let rects = self.rects.clone();
        for r in rects.iter() {
            let mut room = *r;
            if room.x2 as usize == MAPWIDTH - 1 { room.x2 -= 1; }
            // else if room.x2 as usize == MAPWIDTH - 2 { room.x2 -= 2; }
            if room.y2 as usize == MAPHEIGHT - 1 { room.y2 -= 1; }
            // else if room.y2 as usize == MAPHEIGHT - 2 { room.y2 -= 2; }
            self.rooms.push(room);
            apply_room_to_map(&mut self.map, &room);
            self.take_snapshot();
        }

        for i in 0..self.rooms.len()-1 {
            let room = self.rooms[i];
            let next_room = self.rooms[i+1];
            let (start_x, start_y, end_x, end_y) = (
                room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))),
                room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))),
                next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))),
                next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))),
            );
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.take_snapshot();
        }

        let stairs = self.rooms[self.rooms.len()-1].center();
        let stairs_idx = self.map.xy_idx(stairs.0, stairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        let start = self.rooms[0].center();
        self.starting_position = Position{ x: start.0, y: start.1 };
    }

    fn add_subrects(&mut self, rect : Rect, rng : &mut RandomNumberGenerator) {
        if !self.rects.is_empty() { self.rects.remove(self.rects.len() - 1); }

        let (width, height) = (rect.x2 - rect.x1, rect.y2 - rect.y1);
        let (hf_width, hf_height) = (width/2, height/2);

        let split = rng.roll_dice(1,4);

        if split <= 2 { //?  Horizontal split
            let hoz_1 = Rect::new( rect.x1, rect.y1, hf_width-1, height );
            self.rects.push(hoz_1);
            if hf_width > MIN_ROOM_SIZE { self.add_subrects(hoz_1, rng); }
            let hoz_2 = Rect::new( rect.x1+hf_width, rect.y1, hf_width, height );
            self.rects.push(hoz_2);
            if hf_width > MIN_ROOM_SIZE { self.add_subrects(hoz_2, rng); }
        } else { //?  Vertical split
            let ver_1 = Rect::new( rect.x1, rect.y1, width, hf_height-1 );
            self.rects.push(ver_1);
            if hf_height > MIN_ROOM_SIZE { self.add_subrects(ver_1, rng); }
            let ver_2 = Rect::new( rect.x1, rect.y1+hf_height, width, hf_height );
            self.rects.push(ver_2);
            if hf_height > MIN_ROOM_SIZE { self.add_subrects(ver_2, rng); }
        }
    }

    fn draw_corridor(&mut self, x1:i32, y1:i32, x2:i32, y2:i32) {
        let (mut x, mut y) = (x1, y1);

        while x != x2 || y != y2 {
                 if x < x2 { x += 1; }
            else if x > x2 { x -= 1; }
            else if y < y2 { y += 1; }
            else if y > y2 { y -= 1; }

            let idx = self.map.xy_idx(x,y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}
