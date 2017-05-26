extern crate sdl2;

use camera::Camera;
use common::{Vec2, AABB};
use physics::MovingObject;

use std::path::Path;

use sdl2::image::{LoadSurface};
use sdl2::pixels::Color;
use sdl2::rect::Rect;


fn aabb_to_rect(a: &AABB) -> Rect {
  Rect::new(
    (a.center.x - a.half_size.x) as i32,
    (a.center.y - a.half_size.y) as i32,
    (a.half_size.x * 2.) as u32,
    (a.half_size.y * 2.) as u32,
  )
}

pub struct Renderable {
  aabb: AABB,
  texture: sdl2::render::Texture,
  source_rect: sdl2::rect::Rect,
}

impl Renderable {
  pub fn new(renderer: &sdl2::render::Renderer, tex_path: &str, aabb: AABB) -> Renderable {
    let p = Path::new(tex_path);
    let surf = sdl2::surface::Surface::from_file(p).unwrap();
    let texture = renderer.create_texture_from_surface(&surf).unwrap();
    Renderable {
      aabb: aabb,
      texture: texture,
      source_rect: Rect::new(0, 0, surf.width(), surf.height()),
    }
  }

  // Does sprite sheet shifting. Need to write a parameterized version w/ source_rect init as well
  // fn update(&mut self, ticks: u32) {
  //   self.source_rect.set_x((128 * ((ticks / 100) % 6) ) as i32);
  // }
}

pub fn draw(renderer: &mut sdl2::render::Renderer, rend: &Renderable, cam: &Camera) {
  // let dest_rect = rend.source_rect;
  // TODO this is obviously wrong. background needs to be scaled vertically to fit, without squishing X
  let scaling = cam.screen_height / rend.source_rect.height() as f64;
  let dest_rect = Rect::new(0, 0, (rend.source_rect.width() as f64 * scaling) as u32, cam.screen_height as u32);
  renderer.copy(
    &rend.texture, Some(rend.source_rect), Some(dest_rect))
    .unwrap();
}

pub fn draw_physics(object: &MovingObject, renderer: &mut sdl2::render::Renderer, cam: &Camera) {
  let ground_color = Color::RGBA(0, 0, 255, 255);
  let air_color = Color::RGBA(255, 0, 0,  255);
  let draw_color = if object.on_ground { ground_color } else { air_color };

  let ref bb = object.bbox;
  let bl = cam.object2screen(bb.bottom_left(), Vec2::new(0., 0.));
  let tr = cam.object2screen(bb.top_right(), Vec2::new(0., 0.));
  let modified_box = AABB {
    center: cam.object2screen(object.bbox.center, Vec2::new(0., 0.)),
    half_size: Vec2::new(tr.x - bl.x, bl.y - tr.y),
  };
  let draw_rect = aabb_to_rect(&modified_box);
  renderer.set_draw_color(draw_color);
  let _ = renderer.fill_rect(draw_rect);
}
