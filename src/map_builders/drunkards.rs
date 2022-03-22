use rltk::RandomNumberGenerator;
use specs::prelude;
use std::collections::HashMap;
use super::{
    MapBuilder, Map, TileType::*, Position,
    spawner, SHOW_MAPGEN_VISUALIZER,
};

pub struct DrunkardsWalkBuilder {
    map : Map,
    starting_position : Position,
    depth : i32,
    history : Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>,
}

impl map_builders for DrunkardsWalkBuilder {
    fn get_map(&self) -> Map { self.map.clone() }
    fn get_starting_position(&self) -> Position { self.starting_position.clone() }
    fn get_snapshot_history(&self) -> Vec<Map> { self.history.clone() }

    fn build_map(&mut self) { self.build() }

    fn spawn_entities(&mut self, ecs : &mut World) {
        for area in self.noise_areas.iter() {
        spawner::spawn_region(ecs, area.1, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
        let mut snapshot = self.get_map();
        for v in snapshot.revealed_tiles.iter_mut() { *v = true; }
        self.history.push(snapshot);
        }
    }
}

impl DrunkardsWalkBuilder {
    pub fn new(depth : i32) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map : Map::new(new_depth),
            starting_position : Position{ x: 0, y: 0 },
            depth : new_depth,
            history : Vec::new(),
            noise_areas : HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        self.starting_position = Position{ x: map.map_width / 2, y: map.map_height / 2 };
        let start_idx = self.map.xy_idx(self.starting_position.x, self.starting_position.y);

        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height, &map_starts, &self.map, 200.0);
        for (i, tile) in self.map.tiles.iter_mut.enumerate() {
            if *tile == Wall {
                let distance_to_start = dijkstra_map[i];

                if distance_to_start == std::f32::MAX { *tile = Wall }
                else {
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile.0] = DownStairs;
        self.take_snapshot();

        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1 .. self.map_height-1 {
            for x in 1 .. self.map_width-1 {
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