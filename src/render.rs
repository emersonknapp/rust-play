extern crate sdl2;

use camera::Camera;
use common::{Vec2, AABB};
use entity::Entity;
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
  // let scaling = cam.screen_height / rend.source_rect.height() as f64;
  // let dest_rect = Rect::new(0, 0, (rend.source_rect.width() as f64 * scaling) as u32, cam.screen_height as u32);

  let camera_scaling = cam.screen_height / cam.fovy;
  let dest_size = rend.aabb.half_size * camera_scaling * 2.;
  let dest_bottom_left = rend.aabb.center - cam.pos - rend.aabb.half_size;
  let dest_rect = Rect::new(
    dest_bottom_left.x as i32,
    dest_bottom_left.y as i32,
    dest_size.x as u32,
    dest_size.y as u32,
  );

  renderer.copy(
    &rend.texture,
    Some(rend.source_rect),
    Some(dest_rect)
  ).unwrap();
}

pub fn draw_physics(e: &Entity, renderer: &mut sdl2::render::Renderer, cam: &Camera) {
  let ground_color = Color::RGBA(0, 0, 255, 255);
  let air_color = Color::RGBA(255, 0, 0,  255);

  if let Some(ref phys) = e.phys {
    let draw_color = if phys.on_ground { ground_color } else { air_color };

    let world_center = e.center;
    let world_bl = world_center - phys.half_size;
    let world_tr = world_center + phys.half_size;

    let bl = cam.world2screen(world_bl);
    let tr = cam.world2screen(world_tr);
    let draw_rect = Rect::new(
      bl.x as i32,
      tr.y as i32,
      (tr.x - bl.x) as u32,
      (bl.y - tr.y) as u32,
    );

    renderer.set_draw_color(draw_color);
    let _ = renderer.fill_rect(draw_rect);
  }
}
