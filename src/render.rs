extern crate sdl2;

use camera::Camera;
use common::{Vec2, AABB};
use entity::Entity;
use tilemap::Tilemap;

use std::path::Path;

use sdl2::image::{LoadSurface};
use sdl2::pixels::Color;
use sdl2::rect::Rect;


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

pub fn draw(e: &Entity, renderer: &mut sdl2::render::Renderer, cam: &Camera) {
  if let Some(ref rend) = e.rend {
    let bl = e.center + rend.aabb.center - rend.aabb.half_size;
    let draw_rect = cam.to_draw_rect(bl, rend.aabb.half_size * 2.);

    renderer.copy(
      &rend.texture,
      Some(rend.source_rect),
      Some(draw_rect)
    ).unwrap();
  }
}

pub fn draw_physics(e: &Entity, renderer: &mut sdl2::render::Renderer, cam: &Camera) {
  let ground_color = Color::RGBA(0, 0, 255, 255);
  let air_color = Color::RGBA(255, 0, 0,  255);

  if let Some(ref phys) = e.phys {
    let draw_color = if phys.on_ground { ground_color } else { air_color };

    let bl = e.center - phys.half_size;
    let draw_rect = cam.to_draw_rect(bl, phys.half_size * 2.);

    renderer.set_draw_color(draw_color);
    let _ = renderer.fill_rect(draw_rect);
  }
}

pub fn draw_tilemap_collisions(tm: &Tilemap, renderer: &mut sdl2::render::Renderer, cam: &Camera) {
  let ref c = tm.collisions;
  for x in 0..c.nrows() {
    for y in 0..c.ncols() {
      let bl = Vec2::new(x as f64 * 2., y as f64 * 2.);
      let tile_size = Vec2::new(tm.tile_size, tm.tile_size);
      let draw_rect = cam.to_draw_rect(bl, tile_size);

      renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
      let _ = renderer.draw_rect(draw_rect);
    }
  }
}
