extern crate nalgebra as na;
use self::na::{DMatrix};


// TODO: sparse tilemap representation for large maps
// OR: tilemap chunking

pub struct Tilemap {
    pub appearances: DMatrix<u32>,
    pub collisions: DMatrix<bool>,
    pub tile_size: f64,
}

impl Tilemap {
    pub fn new(width: usize, height: usize, tile_size: f64) -> Tilemap {
        Tilemap {
            tile_size: tile_size,
            appearances: DMatrix::from_element(width, height, 0),
            collisions: DMatrix::from_element(width, height, false),
        }
    }
}
