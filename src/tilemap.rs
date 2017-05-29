extern crate nalgebra as na;
use self::na::{DMatrix};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;


// TODO: sparse tilemap representation for large maps
// OR: tilemap chunking

pub struct Tilemap {
  pub appearances: DMatrix<u32>,
  pub collisions: DMatrix<bool>,
  pub tile_size: f64,
  pub width: usize,
  pub height: usize,
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
}
