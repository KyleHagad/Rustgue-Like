use specs::prelude::*;
use super::{
  Map, Rect, TileType, Position,
  spawner,
};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use common::*;

pub trait MapBuilder {
  fn build_map(&mut self);
  fn spawn_entities(&mut self, ecs : &mut World);
  fn get_map(&mut self) -> Map;
  fn get_starting_position(&mut self) -> Position;
}

pub fn random_builder(new_depth : i32) -> Box<dyn MapBuilder> {
  Box::new(SimpleMapBuilder::new(new_depth))
}
