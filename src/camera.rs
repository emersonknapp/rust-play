use common::{Vec2};

use std::f64;
use sdl2::rect::Rect;

pub struct Camera {
    pub fovy: f64,
    pub ratio: f64, // fovy * ratio = fovx
    pub screen_height: f64,
    pub pos: Vec2,
}
impl Camera {
  pub fn to_draw_rect(&self, bl: Vec2, size: Vec2) -> Rect {
    let fovx = self.ratio * self.fovy;
    let u2s = self.screen_height as f64 / self.fovy;
    let half_size = Vec2::new(fovx / 2., self.fovy / 2.);

    let camera_bl = bl - self.pos + half_size;
    let camera_y = camera_bl.y + size.y;
    let screen_y = self.screen_height - (camera_y * u2s);

    let screen_x = camera_bl.x * u2s;

    // round coordinates down so they don't have gaps between,
    // and round size up to err towards overlap adjacent tiles
    Rect::new(
      screen_x.floor() as i32,
      screen_y.floor() as i32,
      (size.x * u2s).ceil() as u32,
      (size.y * u2s).ceil() as u32,
    )
  }

  pub fn screen2world(&self, x: i32, y: i32) -> Vec2 {
    let fovx = self.ratio * self.fovy;
    let u2s = self.screen_height as f64 / self.fovy;
    let half_size = Vec2::new(fovx / 2., self.fovy / 2.);

    let camera_x = x as f64 / u2s;
    let world_x = camera_x - half_size.x + self.pos.x;

    let camera_y = (self.screen_height as f64 - y as f64) / u2s;
    let world_y = camera_y - half_size.y + self.pos.y;

    Vec2::new(world_x, world_y)
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut cam = Camera {
          fovy: 10.,
          screen_height: 100.,
          pos: Vec2::new(0., 0.)
        };

        let draw_rect = cam.to_draw_rect(Vec2::new(0., 0.), Vec2::new(1., 1.));
        assert!(draw_rect.left() == 0);
        assert!(draw_rect.top() == 90);

        let dr2 = cam.to_draw_rect(Vec2::new(2., 3.), Vec2::new(1., 2.));
        assert!(dr2.left() == 20);
        assert!(dr2.top() == 50);

        cam.pos = Vec2::new(4., 3.);
        let dr3 = cam.to_draw_rect(Vec2::new(1., 1.), Vec2::new(1., 1.));
        assert!(dr3.left() == -30);
        assert!(dr3.top() == 110);
    }
}
