extern crate nalgebra as na;

use common::{Vec2, AABB};

// px/sec^2
const GRAVITY: f64 = 600.;

pub struct MovingObject {
  pub pos: Vec2,
  pub speed: Vec2,
  pub bbox: AABB,
  pub on_ground: bool,
}

impl MovingObject {
  pub fn update(&mut self, dt_seconds: f64) {
    self.pos += self.speed * dt_seconds;
    self.speed.y -= GRAVITY * dt_seconds;

    if self.pos.y <= self.bbox.half_size.y {
      self.pos.y = self.bbox.half_size.y;
      self.speed.y = 0.;
      self.on_ground = true;
    } else {
      self.on_ground = false;
    }

    self.bbox.center = self.pos;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn ctor() {
    let a = AABB::new(Vec2::new(0., 0.), Vec2::new(1., 1.));
  }
  #[test]
  fn intersections() {
    let a = AABB::new(Vec2::new(0., 0.), Vec2::new(1., 1.));
    let b = AABB::new(Vec2::new(5., 5.), Vec2::new(1., 1.));
    assert!(!a.intersects(&b));

    let c = AABB::new(Vec2::new(1., 1.), Vec2::new(1., 1.));
    assert!(a.intersects(&c));
  }
}
