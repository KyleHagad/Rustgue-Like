use rltk::{ Rltk, RGB, BaseMap, Algorithm2D, Point };
use serde::{ Serialize, Deserialize };
use std::collections::HashSet;
use specs::prelude::*;
use super::{ Rect };

pub const MAPHEIGHT : usize = 43;
pub const MAPWIDTH : usize = 80;
pub const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub depth : i32,
    pub bloodstains : HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content : Vec<Vec<Entity>>
}
impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x,y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn new(new_depth : i32) -> Map {
        Map{
            tiles : vec![TileType::Wall; MAPCOUNT],
            width : MAPWIDTH as i32,
            height : MAPHEIGHT as i32,
            revealed_tiles : vec![false; MAPCOUNT],
            visible_tiles : vec![false; MAPCOUNT],
            blocked : vec![false; MAPCOUNT],
            tile_content : vec![Vec::new(); MAPCOUNT],
            depth : new_depth,
            bloodstains : HashSet::new(),
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x,y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1,p2)
    }

    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinals
        if self.is_exit_valid(x-1,y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1,y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x,y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x,y+1) { exits.push((idx+w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut x = 0;
    let mut y = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {

        if map.revealed_tiles[idx] {
            let mut glyph;
            let mut fg;
            let mut bg = RGB::named(rltk::BLACK);

            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('·');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x,y);
                    fg = RGB::named(rltk::VIOLET);
                },
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('»');
                    fg = RGB::from_f32(0., 1.0, 1.0);
                }
            }
            if map.bloodstains.contains(&idx) {
                fg = RGB::named(rltk::DARKRED);
                bg = RGB::named(rltk::RED);
                glyph = rltk::to_cp437('░');
            };
            if !map.visible_tiles[idx] { fg = fg.to_greyscale(); bg = bg.to_greyscale(); }
            ctx.set(x,y, fg, bg, glyph);
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

fn wall_glyph(map : &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 as i32 { return 219; }
    let mut mask : u8 = 0;

    if is_revealed_and_wall(map, x,y-1) { mask += 1; }
    if is_revealed_and_wall(map, x,y+1) { mask += 2; }
    if is_revealed_and_wall(map, x-1,y) { mask += 4; }
    if is_revealed_and_wall(map, x+1,y) { mask += 8; }

    match mask {
        0 => { 9 } // Pillar
        1 => { 186 } // Wall to north
        2 => { 186 } // wall to south
        3 => { 186 } // wall to north and south
        4 => { 205 } // wall to west
        5 => { 188 } // wall to north and west
        6 => { 187 } // wall to south and west
        7 => { 185 } // wall to north, south and west
        8 => { 205 } // wall to east
        9 => { 200 } // wall to north and east
        10 => { 201 } // wall to south and east
        11 => { 204 } // wall to north, south and east
        12 => { 205 } // wall to east and west
        13 => { 202 } // wall to south, east and west
        14 => { 203 } // wall to north, west and east
        15 => { 206 } // wall to all directions
        _ => { 219 } // missed one?
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x,y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
