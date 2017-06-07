extern crate sdl2;

use camera::Camera;
use common::{Vec2, Vec2u, AABB};
use tilemap::Tilemap;
use components::{Position, Collision};

use std::path::Path;

use sdl2::image::{LoadSurface};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

pub struct Sprite {
  aabb: AABB,
  texture: sdl2::render::Texture,
  source_rect: sdl2::rect::Rect,
}

impl Sprite {
  pub fn new(renderer: &Renderer, tex_path: &str, aabb: AABB) -> Sprite {
    let p = Path::new(tex_path);
    let surf = sdl2::surface::Surface::from_file(p).unwrap();
    let texture = renderer.create_texture_from_surface(&surf).unwrap();
    Sprite {
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

pub fn draw(sprite: &Sprite, pos: &Position, renderer: &mut Renderer, cam: &Camera) {
    let bl = pos + sprite.aabb.center - sprite.aabb.half_size;
    let draw_rect = cam.to_draw_rect(bl, sprite.aabb.half_size * 2.);

    let _ = renderer.copy(
      &sprite.texture,
      Some(sprite.source_rect),
      Some(draw_rect)
    );
}

pub fn draw_physics(position: &Position, collision: &Collision, on_ground: bool, renderer: &mut Renderer, cam: &Camera) {
  let ground_color = Color::RGBA(0, 0, 255, 255);
  let air_color = Color::RGBA(255, 0, 0,  255);
  let draw_color = if on_ground { ground_color } else { air_color };

  let bl = collision.bottom_left() + position;
  let draw_rect = cam.to_draw_rect(bl, collision.half_size * 2.);

  renderer.set_draw_color(draw_color);
  let _ = renderer.fill_rect(draw_rect);
}

pub fn draw_tilemap_collisions(tm: &Tilemap, intersected: &Vec<Vec2u>, renderer: &mut Renderer, cam: &Camera) {
  let ref c = tm.collisions;
  // Draw tilemap collision layer
  for y in 0..c.nrows() {
    for x in 0..c.ncols() {
      let draw_color;
      if c[(y, x)] {
        draw_color = Color::RGBA(200, 140, 0, 255);
      } else {
        continue;
      }
      let bl = Vec2::new(x as f64 * tm.tile_size, y as f64 * tm.tile_size);
      let tile_size = Vec2::new(tm.tile_size, tm.tile_size);
      let draw_rect = cam.to_draw_rect(bl, tile_size);
      renderer.set_draw_color(draw_color);
      let _ = renderer.fill_rect(draw_rect);
    }
  }
  // Debug draw collisions with the tilemap
  for i in intersected {
    let draw_color = Color::RGBA(0, 255, 0, 200);
    let bl = Vec2::new(i.x as f64 * tm.tile_size, i.y as f64 * tm.tile_size);
    let tile_size = Vec2::new(tm.tile_size, tm.tile_size);
    let draw_rect = cam.to_draw_rect(bl, tile_size);
    renderer.set_draw_color(draw_color);
    let _ = renderer.fill_rect(draw_rect);
  }
}
//
// pub fn draw_tile(renderer: &mut Renderer, camera: &Camera, coord: (i32, i32), tile_size: f64) {
//   let draw_color = Color::RGBA(0, 255, 255, 255);
//   let bl = Vec2::new(coord.0 as f64 * tile_size, coord.1 as f64 * tile_size);
//   let tsvec = Vec2::new(tile_size, tile_size);
//   let draw_rect = camera.to_draw_rect(bl, tsvec);
//   renderer.set_draw_color(draw_color);
//   let _ = renderer.fill_rect(draw_rect);
// }
