extern crate nalgebra as na;
extern crate sdl2;

use std::collections::HashSet;
use self::sdl2::keyboard::{Keycode, Mod};
use self::sdl2::mouse::{MouseState, MouseButton};

pub type Vec2 = self::na::Vector2<f64>;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

#[derive(Serialize, Deserialize, Debug)]
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
  // pub fn top_right(&self) -> Vec2 {
  //   self.center + self.half_size
  // }
  // pub fn bottom_right(&self) -> Vec2 {
  //   Vec2::new(self.center.x + self.half_size.x, self.center.y - self.half_size.y)
  // }
  // pub fn top_left(&self) -> Vec2 {
  //   Vec2::new(self.center.x - self.half_size.x, self.center.y + self.half_size.y)
  // }
  // pub fn intersects(&self, other: &AABB) -> bool {
  //   ! (
  //     ((self.center.x - other.center.x).abs() > (self.half_size.x + other.half_size.x)) ||
  //     ((self.center.y - other.center.y).abs() > self.half_size.y + other.half_size.y)
  //   )
  // }

  pub fn intersect(&self, other: &AABB) -> Option<Vec2> {
    // x is collision on the left, -x on the right
    // y bottom collision, -y top
    let dv = {
      let dsigned = other.center - self.center;
      Vec2::new(dsigned.x.abs(), dsigned.y.abs())
    };
    let combined_size = self.half_size + other.half_size;

    let mut x_overlap = None;
    let mut y_overlap = None;
    if dv.x < combined_size.x {
      // there is an x overlap
      let on_right = self.center.x > other.center.x;
      x_overlap = Some(
        (dv.x - combined_size.x) *
        (if on_right { -1. } else { 1. })
      );
    }
    if dv.y < combined_size.y {
      let on_top = self.center.y > other.center.y;
      y_overlap = Some(
        (dv.y - combined_size.y) *
        (if on_top { -1. } else { 1. })
      );
    }

    match (x_overlap, y_overlap) {
      (Some(xo), Some(yo)) => Some(Vec2::new(xo, yo)),
      _ => None
    }
  }
}

pub struct InputState {
  pub keys: HashSet<Keycode>,
  pub last_keys: HashSet<Keycode>,
  pub key_mod: Mod,
  pub mouse: MouseState,
  pub last_mouse: MouseState,
}

impl InputState {
  pub fn key_down(&self, k: &Keycode) -> bool {
    self.keys.contains(k)
  }
  pub fn key_pressed(&self, k: &Keycode) -> bool {
    self.keys.contains(k) && !self.last_keys.contains(k)
  }
  // pub fn key_released(&self, k: &Keycode) -> bool {
  //   !self.keys.contains(k) && self.last_keys.contains(k)
  // }
  pub fn mouse_pressed(&self, b: MouseButton) -> bool {
    self.mouse.is_mouse_button_pressed(b) && !self.last_mouse.is_mouse_button_pressed(b)
  }
  pub fn mouse_down(&self, b: MouseButton) -> bool {
    self.mouse.is_mouse_button_pressed(b)
  }
}
