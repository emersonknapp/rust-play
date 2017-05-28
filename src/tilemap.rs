extern crate nalgebra as na;
use self::na::{DMatrix};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io;
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

    let mut width: usize = 0;
    let mut height: usize = 0;
    let mut tile_size: f64 = 0.;
    let mut got_rows = 0;
    let mut printed_warning = false;

    for (idx, line) in reader.lines().enumerate() {
      let l = line.unwrap();
      let tokens: Vec<&str> = l.split_whitespace().collect();
      if idx == 0 {
        width = tokens[0].parse().unwrap();
        height = tokens[1].parse().unwrap();
        tile_size = tokens[2].parse().unwrap();
        println!("Got TM desc: {} {} {}", width, height, tile_size);
        continue;
      }

      match tokens.len() {
        0 => continue,
        num if num >= width => {
          if num > width && !printed_warning {
            println!("Warning: level width truncated");
            printed_warning = true;
          }
          let values = tokens.iter().map(|&v| v == "0");
          got_rows += 1;
        },
        default => panic!("Line too short, {}", default),
      }

    }
    assert!(got_rows == height);

    Ok(Tilemap::new(width, height, tile_size))
  }
}
