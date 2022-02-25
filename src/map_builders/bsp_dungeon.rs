use rltk::RandomNumberGenerator;
use specs::prelude::*;
use super::{
    Map, Position, Rect, MapBuilder, TileType,
    spawner, apply_room_to_map,
    SHOW_MAPGEN_VISUALIZER,
};

pub struct BspDungeonBuilder {
    map : Map,
    starting_position : Position,
    depth : i32,
    rooms : Vec<Rect>,
    history : Vec<Map>,
    rects : Vec<Rect>,
}

impl MapBuilder for BspDungeonBuilder {
    fn get_map(&mut self) -> Map { self.map.clone() }

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

impl BspDungeonBuilder {
    pub fn new(new_depth : i32) -> BspDungeonBuilder {
        BspDungeonBuilder{
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
        self.rects.push( Rect::new(2,2, self.map.width-5, self.map.height-3) );
        let first_room = self.rects[0];
        self.add_subrects(first_room);

        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(&mut rng);
            let candidate = self.get_random_sub_rect(rect, &mut rng);

            if self.is_possible(candidate) {
                apply_room_to_map(&mut self.map, &candidate);
                self.rooms.push(candidate);
                self.add_subrects(rect);
                self.take_snapshot();
            }

            n_rooms += 1;
        }

        self.rooms.sort_by( |a,b| a.x1.cmp(&b.x1) );

        for i in 0..self.rooms.len() -1 {
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

    fn add_subrects(&mut self, rect : Rect) {
        let (width, height) = ( i32::abs(rect.x1 - rect.x2), i32::abs(rect.y1 - rect.y2), );
        let (hf_width, hf_height) = ( i32::max(width / 2, 1), i32::max(height / 2, 1), );

        self.rects.push(Rect::new( rect.x1, rect.y1, hf_width, hf_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + hf_height, hf_width, hf_height ));
        self.rects.push(Rect::new( rect.x1 + hf_width, rect.y1, hf_width, hf_height ));
        self.rects.push(Rect::new( rect.x1 + hf_width, rect.y1 + hf_height, hf_width, hf_height ));
    }

    fn get_random_rect(&mut self, rng : &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);

        let (w,h) = (
            i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) +1,
            i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) +1,
        );

        result.x1 += rng.roll_dice(1, 6) -1;
        result.y1 += rng.roll_dice(1, 6) -1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;

        result
    }

    fn is_possible(&self, rect : Rect) -> bool {
        let mut expanded = rect;
        expanded.x1 -=2;
        expanded.x2 +=2;
        expanded.y1 -=2;
        expanded.y2 +=2;

        let mut can_build = true;

        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > self.map.width-2 { can_build = false; }
                if y > self.map.height-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = self.map.xy_idx(x,y);
                    if self.map.tiles[idx] != TileType::Wall {
                        can_build = false;
                    }
                }
            }
        }

        can_build
    }

    fn draw_corridor(&mut self, x1:i32, y1:i32, x2:i32, y2:i32) {
        let (mut x, mut y) = (x1,y1);
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
