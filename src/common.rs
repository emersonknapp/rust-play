extern crate nalgebra as na;

pub type Vec2 = self::na::Vector2<f64>;
pub type Vec2u = self::na::Vector2<usize>;

pub struct AABB {
  pub center: Vec2,
  pub half_size: Vec2,
}
impl AABB {
  pub fn new(center: Vec2, half_size: Vec2) -> AABB {
    AABB {
      center: center,
      half_size: half_size,
    }
  }
  pub fn offset(&self, v: Vec2) -> AABB {
    AABB {
      center: self.center + v,
      half_size: self.half_size,
    }
  }
  pub fn bottom_left(&self) -> Vec2 {
    self.center - self.half_size
  }
  pub fn top_right(&self) -> Vec2 {
    self.center + self.half_size
  }
  pub fn bottom_right(&self) -> Vec2 {
    Vec2::new(self.center.x + self.half_size.x, self.center.y - self.half_size.y)
  }
  pub fn top_left(&self) -> Vec2 {
    Vec2::new(self.center.x - self.half_size.x, self.center.y + self.half_size.y)
  }
  pub fn intersects(&self, other: &AABB) -> bool {
    ! (
      ((self.center.x - other.center.x).abs() > (self.half_size.x + other.half_size.x)) ||
      ((self.center.y - other.center.y).abs() > self.half_size.y + other.half_size.y)
    )
  }
}
