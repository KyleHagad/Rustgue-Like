use specs::prelude::*;
use super::{
  Map, Rect, TileType, Position, spawner,
};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

trait MapBuilder {
  fn build(new_depth: i32) -> (Map, Position);
  fn spawn(map : &mut Map, ecs : &mut World, new_depth: i32);
}

pub fn build_random_map(new_depth: i32) -> (Map, Position) {
  SimpleMapBuilder::build(new_depth)
}

pub fn spawn(map : &mut Map, ecs : &mut World, new_depth: i32) {
  SimpleMapBuilder::spawn(map, ecs, new_depth);
}
