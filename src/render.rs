extern crate sdl2;

use camera::Camera;
use common::{Vec2, Vec2u, AABB};
use components::{Position, Collision, World, DrawObstacleTool};

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

pub fn draw_rect(renderer: &mut Renderer, camera: &Camera, bl: Vec2, size: Vec2, color: Color) {
  let rect = camera.to_draw_rect(bl, size);
  renderer.set_draw_color(color);
  let _ = renderer.fill_rect(rect);
}

pub fn draw_physics(position: &Position, collision: &Collision, on_ground: bool, renderer: &mut Renderer, cam: &Camera) {
  let draw_color = if on_ground {
    Color::RGBA(0, 0, 255, 255)
  } else {
    Color::RGBA(255, 0, 0,  255)
  };
  draw_rect(renderer, cam, collision.offset(*position).bottom_left(), collision.half_size * 2., draw_color);
}

pub fn draw_static(position: &Position, collision: &Collision, renderer: &mut Renderer, cam: &Camera, is_collided: bool) {
  let draw_color = if is_collided { Color::RGBA(0, 255, 255, 255) } else { Color::RGBA(255, 255, 0, 255) };
  draw_rect(renderer, cam, collision.offset(*position).bottom_left(), collision.half_size * 2., draw_color);
}


pub fn draw_obstacle_tool(tool: &DrawObstacleTool, camera: &Camera, renderer: &mut Renderer) {
  if let Some(start_pos) = tool.start_pos {
    draw_rect(renderer, camera,
      Vec2::new(start_pos.x.min(tool.pos.x), start_pos.y.min(tool.pos.y)),
      (start_pos - tool.pos).abs(),
      Color::RGBA(90, 150, 20, 120),
    );
  }

  let dot_size = Vec2::new(0.5, 0.5);
  draw_rect(renderer, camera, tool.pos - dot_size / 2., dot_size, Color::RGBA(255, 0, 255, 255));

}

pub fn render_system(world: &World, renderer: &mut Renderer) {
  if let Some(ref camera) = world.cameras.get(&world.current_camera) {
    //TODO render needs drawing order (z coord?)
    for id in &world.entities {
      if let (Some(ref sprite), Some(ref pos)) =
             (world.sprites.get(&id), world.positions.get(&id))
      {
        draw(sprite, pos, renderer, camera);
      }
    }
    for id in &world.entities {
      if let (Some(ref collision), Some(ref position), None) =
             (world.collisions.get(&id), world.positions.get(&id), world.velocities.get(&id))
      {
        draw_static(position, collision, renderer, camera, world.statics_collisions.contains(id));
      }
    }
    for id in &world.entities {
      if let (Some(ref collision), Some(ref position), Some(on_ground)) =
             (world.collisions.get(&id), world.positions.get(&id), world.groundables.get(&id))
      {
        draw_physics(position, collision, *on_ground, renderer, camera);
      }
      if let Some(obstool) = world.obstacle_tools.get(&id) {
        draw_obstacle_tool(obstool, camera, renderer);
      }
    }
  }
}
