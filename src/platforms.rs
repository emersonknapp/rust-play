extern crate nalgebra;
use self::nalgebra::{distance, dot, norm};

use common::{Vec2};
use components::{World, Position};

#[derive(Serialize, Deserialize, Debug)]
pub struct MoverBlock {
  pub start: Vec2,
  pub end: Vec2,
  pub travel_time: f64,
}

fn mover_block(pos: &mut Position, mover: &MoverBlock, dt_seconds: f64) {
  // travel_time is from 0 (start) to 1 (end) in seconds
  // map pos to the line, update appropriately, and output next position based on that
  let dnorm = dt_seconds / mover.travel_time;

  let a = *pos - mover.start;
  let alen = a.norm();
  let b = mover.end - mover.start;
  let blen = b.norm();

  let cur_norm = alen / blen;
  let mut next_norm = cur_norm + dnorm;
  if next_norm > 1. {
    next_norm -= 1.;
  }
  *pos = (next_norm * b) + mover.start;
}

pub fn mover_blocks(world: &mut World, dt_seconds: f64) {
  for (id, block) in &world.mover_blocks {
    if let Some(ref mut p) = world.positions.get_mut(id) {
      mover_block(p, block, dt_seconds);
    }
  }
}
