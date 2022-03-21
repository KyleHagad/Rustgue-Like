use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;
use super::{
    MapBuilder, Map, Position, TileType::*,
    spawner,
    SHOW_MAPGEN_VISUALIZER,
};

pub struct CellularAutomataBuilder {
    map : Map,
    starting_position : Position,
    depth : i32,
    history : Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
    fn get_map(&self) -> Map { self.map.clone() }
    fn get_starting_position(&self) -> Position { self.starting_position.clone() }
    fn get_snapshot_history(&self) -> Vec<Map> { self.history.clone() }

    fn build_map(&mut self) { self.build(); }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for area in self.noise_areas.iter() {
            spawner::spawn_region(ecs, area.1, self.depth);
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

impl CellularAutomataBuilder {
    pub fn new(new_depth : i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder{
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        // Make some noise
        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let roll = rng.roll_dice(1,100);
                let idx = self.map.xy_idx(x,y);
                if roll > 55 { self.map.tiles[idx] = Floor }
                else { self.map.tiles[idx] = Wall }
            }
        }
        self.take_snapshot();
        // Iterate cell rules over the noise
        for _i in 0..15 {
            let mut new_tiles = self.map.tiles.clone();

            for y in 1..self.map.height-1 {
                for x in 1..self.map.width-1 {
                    let idx = self.map.xy_idx(x,y);
                    let mut neighbors = 0;

                    if self.map.tiles[idx-1] == Wall { neighbors += 1; } // To the left
                    if self.map.tiles[idx+1] == Wall { neighbors += 1; } // To the right
                    if self.map.tiles[idx - self.map.width as usize] == Wall { neighbors += 1; } // Above
                    if self.map.tiles[idx + self.map.width as usize] == Wall { neighbors += 1; } // Below
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == Wall { neighbors += 1; } // Above left
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == Wall { neighbors += 1; } // Above right
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == Wall { neighbors += 1; } // Below left
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == Wall { neighbors += 1; } // Below right

                    if neighbors > 4 || neighbors == 0 { new_tiles[idx] = Wall; }
                    else { new_tiles[idx] = Floor; }
                }
            }

            self.map.tiles = new_tiles.clone();
            self.take_snapshot();
        }

        self.starting_position = Position{ x: self.map.width / 2, y: self.map.height / 2 };
        let mut start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != Floor {
            self.starting_position.x -= 1;
            start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);
        }
        self.take_snapshot();

        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(self.map.width as usize, self.map.height as usize, &map_starts, &self.map, 200.0);
        let mut exit_tile = (0, 0.0f32);
        // let mut distances: Vec<f32> = Vec::new();
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == Floor {
                let distance_to_start = dijkstra_map.map[i];
                // distances.push(distance_to_start);
                if distance_to_start == f32::MAX { *tile = Wall; }
                else {
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        // distances.sort_by(|a,b| a.partial_cmp(b).unwrap());
        // rltk::log(format!("{:?}", distances[distances.len()-1]));
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = DownStairs;
        self.take_snapshot();

        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let idx = self.map.xy_idx(x,y);
                if self.map.tiles[idx] == Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if self.noise_areas.contains_key(&cell_value) {
                        self.noise_areas.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.noise_areas.insert(cell_value, vec![idx]);
                    }
                }
            }
        }
    }
}
