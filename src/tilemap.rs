extern crate nalgebra as na;
extern crate sdl2;
use self::sdl2::mouse::MouseButton;
use self::na::{DMatrix};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use camera::Camera;

use common::{Vec2, Vec2u, AABB, InputState};

// TODO: sparse tilemap representation for large maps
// OR: tilemap chunking

pub struct Tilemap {
  pub appearances: DMatrix<u32>,
  pub collisions: DMatrix<bool>,
  pub tile_size: f64,
  pub width: usize,
  pub height: usize,
}

enum TilemapAction {
  ToggleTileCollision(usize, usize),
}


impl Tilemap {
  pub fn new(width: usize, height: usize, tile_size: f64) -> Tilemap {
    Tilemap {
      tile_size: tile_size,
      width: width,
      height: height,
      appearances: DMatrix::from_element(height, width, 0),
      collisions: DMatrix::from_element(height, width, false),
    }
  }

  pub fn from_file(p: &Path) -> Result<Tilemap, String> {
    let file = File::open(p).unwrap();
    let reader = BufReader::new(&file);

    // read one line, gives width, height, tile_size
    // then, read lines, number of tokens `width`
    // read `height` lines

    let mut got_rows = 0;
    let mut printed_warning = false;
    let mut tiles = Tilemap::new(0, 0, 0.);

    for (idx, line) in reader.lines().enumerate() {
      let l = line.unwrap();
      let tokens: Vec<&str> = l.split_whitespace().collect();
      if idx == 0 {
        let width = tokens[0].parse().unwrap();
        let height = tokens[1].parse().unwrap();
        tiles = Tilemap {
          width: width,
          height: height,
          tile_size: tokens[2].parse().unwrap(),
          appearances: DMatrix::from_element(height, width, 0),
          collisions: DMatrix::from_element(height, width, false),
        };
        println!("Got TM desc: {} {} {}", width, height, tiles.tile_size);
        continue;
      }

      match tokens.len() {
        0 => continue,
        num if num < tiles.width => panic!("Row length didn't match level desc"),
        num if num >= tiles.width => {
          if num > tiles.width && !printed_warning {
            println!("Warning: level width truncated");
            printed_warning = true;
          }
          let values = tokens.iter();
          for (index, &v) in values.enumerate() {
            let tile_type: usize = v.parse().unwrap();
            if tile_type > 0 {
              tiles.collisions[(tiles.height - got_rows - 1, index)] = true;
            }
          }
          got_rows += 1;
        },
        default => panic!("Line too short, {}", default),
      }
    }
    assert!(got_rows == tiles.height);

    Ok(tiles)
  }

  fn rightmost(&self) -> f64 {
    self.tile_size * (self.width + 1) as f64
  }

  fn topmost(&self) -> f64 {
    self.tile_size * (self.height + 1) as f64
  }

  pub fn tile_for(&self, local_coord: Vec2) -> Option<(usize, usize)> {
    if local_coord.x < 0. || local_coord.y < 0. {
      return None;
    }
    let x = (local_coord.x / self.tile_size).floor();
    let y = (local_coord.y / self.tile_size).floor();
    if x as usize >= self.width || y as usize >= self.height {
      return None;
    }
    Some((x as usize, y as usize))
  }

  fn get_tile(&self, x: usize, y: usize) -> bool {
    if x >= self.width || y >= self.height {
      return false;
    }
    self.collisions[(y, x)]
  }

  pub fn intersects_box(&self, aabb: &AABB) -> Vec<Vec2u> {
    let mut isects = Vec::new();
    let obl = self.tile_for(aabb.bottom_left());
    let otr = self.tile_for(aabb.top_right());
    // TODO: this is wrong on the edges, needs to clamp
    if let (Some(bl), Some(tr)) = (obl, otr) {
      for x in bl.0..(tr.0+1) {
        for y in bl.1..(tr.1+1) {
          if self.get_tile(x, y) {
            isects.push(Vec2u::new(x, y));
          }
        }
      }
    }
    isects
  }

  fn resolve_action(&mut self, action: &TilemapAction) {
    match action {
      &TilemapAction::ToggleTileCollision(x, y) => {
        self.collisions[(y, x)] = !self.collisions[(y, x)]
      },
    }
  }

  pub fn input(&mut self, input: &InputState, camera: &Camera) {
    if input.mouse_pressed(MouseButton::Left) {
      let world_coord = camera.screen2world(input.mouse.x(), input.mouse.y());
      if let Some((x, y)) = self.tile_for(world_coord) {
        self.resolve_action(&TilemapAction::ToggleTileCollision(x, y));
      }
    }
  }

}
