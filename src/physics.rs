use common::{Vec2, AABB};

// px/sec^2
const GRAVITY: f64 = 600.;

pub struct MovingObject {
  pub speed: Vec2,
  pub aabb: AABB,
  pub on_ground: bool,
}

impl MovingObject {
  pub fn update(&mut self, center: Vec2, dt_seconds: f64) -> Vec2 {
    let dpos = self.speed * dt_seconds;
    let mut next = center + dpos;

    self.speed.y -= GRAVITY * dt_seconds;

    if next.y <= self.aabb.half_size.y {
      next.y = self.aabb.half_size.y;
      self.speed.y = 0.;
      self.on_ground = true;
    } else {
      self.on_ground = false;
    }
    next
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use common::AABB;
  #[test]
  fn ctor() {
    let _ = AABB::new(Vec2::new(0., 0.), Vec2::new(1., 1.));
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
