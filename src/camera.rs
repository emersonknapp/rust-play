use common::{Vec2};

use std::f64;
use sdl2::rect::Rect;

pub struct Camera {
    pub fovy: f64,
    pub screen_height: f64,
    pub pos: Vec2,
}
impl Camera {
  pub fn to_draw_rect(&self, bl: Vec2, size: Vec2) -> Rect {
    let bl = bl - self.pos;
    let u2s = self.screen_height as f64 / self.fovy;

    // round coordinates down so they don't spread, but round size up to overlap/pad adjacent tiles
    Rect::new(
      (bl.x * u2s).floor() as i32,
      (self.screen_height - ((bl.y + size.y) * u2s)).floor() as i32,
      (size.x * u2s).ceil() as u32,
      (size.y * u2s).ceil() as u32,
    )

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
