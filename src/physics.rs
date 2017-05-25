#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
extern crate nalgebra as na;

pub type vec2 = self::na::Vector2<f64>;

pub struct AABB {
  pub center: vec2,
  pub halfSize: vec2,
}
impl AABB {
    pub fn bottom_left(&self) -> vec2 {
        self.center - self.halfSize
    }
    pub fn top_right(&self) -> vec2 {
        self.center + self.halfSize
    }
}

// px/sec^2
const GRAVITY: f64 = 600.;

impl AABB {
  pub fn new(center: vec2, halfSize: vec2) -> AABB {
    AABB {
      center: center,
      halfSize: halfSize,
    }
  }
  pub fn intersects(&self, other: &AABB) -> bool {
    ! (
      ((self.center.x - other.center.x).abs() > (self.halfSize.x + other.halfSize.x)) ||
      ((self.center.y - other.center.y).abs() > self.halfSize.y + other.halfSize.y)
    )
  }
}

pub struct MovingObject {
  pub pos: vec2,
  pub speed: vec2,
  pub bbox: AABB,
  pub onGround: bool,
}

impl MovingObject {
  pub fn update(&mut self, dt_seconds: f64) {
    self.pos += self.speed * dt_seconds;
    self.speed.y -= GRAVITY * dt_seconds;

    if self.pos.y <= self.bbox.halfSize.y {
      self.pos.y = self.bbox.halfSize.y;
      self.speed.y = 0.;
      self.onGround = true;
    } else {
      self.onGround = false;
    }

    self.bbox.center = self.pos;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn ctor() {
    let a = AABB::new(vec2::new(0., 0.), vec2::new(1., 1.));
  }
  #[test]
  fn intersections() {
    let a = AABB::new(vec2::new(0., 0.), vec2::new(1., 1.));
    let b = AABB::new(vec2::new(5., 5.), vec2::new(1., 1.));
    assert!(!a.intersects(&b));

    let c = AABB::new(vec2::new(1., 1.), vec2::new(1., 1.));
    assert!(a.intersects(&c));
  }
}
