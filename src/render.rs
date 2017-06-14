extern crate sdl2;

use camera::Camera;
use common::{Vec2, AABB};
use components::{Position, Collision, World};

use std::path::Path;
use std::fmt;

use sdl2::image::{LoadSurface};
use sdl2::pixels::Color;
use sdl2::render::Renderer;


#[derive(Serialize, Deserialize, Debug)]
struct Rect {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
}

impl Rect {
  fn new(x: i32, y: i32, width: u32, height: u32) -> Rect {
    Rect { x: x, y: y, width: width, height: height }
  }
  fn to_sdl_rect(&self) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(self.x, self.y, self.width, self.height)
  }
}

#[derive(Serialize, Deserialize)]
pub struct Sprite {
  aabb: AABB,
  source_rect: Rect,
  source_path: String,
  #[serde(skip)]
  texture: Option<sdl2::render::Texture>,
}

fn load_texture(path: &str, renderer: &Renderer) -> (sdl2::render::Texture, u32, u32) {
  let p = Path::new(path);
  let surf = sdl2::surface::Surface::from_file(p).unwrap();
  (renderer.create_texture_from_surface(&surf).unwrap(), surf.width(), surf.height())
}

impl Sprite {
  pub fn new(renderer: &Renderer, tex_path: &str, aabb: AABB) -> Sprite {
    let (tex, width, height) = load_texture(tex_path, renderer);
    Sprite {
      aabb: aabb,
      source_rect: Rect::new(0, 0, width, height),
      source_path: tex_path.to_owned(),
      texture: Some(tex),
    }
  }

  pub fn reload_assets(&mut self, renderer: &Renderer) {
    let (tex, width, height) = load_texture(&self.source_path[..], renderer);
    self.source_rect = Rect::new(0, 0, width, height);
    self.texture = Some(tex);
  }

  // Does sprite sheet shifting. Need to write a parameterized version w/ source_rect init as well
  // fn update(&mut self, ticks: u32) {
  //   self.source_rect.set_x((128 * ((ticks / 100) % 6) ) as i32);
  // }
}

impl fmt::Debug for Sprite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sprite {{ {:?}, {:?}, {} }}", self.aabb, self.source_rect, self.source_path)
    }
}

pub fn draw_rect(renderer: &mut Renderer, camera: &Camera, bl: Vec2, size: Vec2, color: Color) {
  let rect = camera.to_draw_rect(bl, size);
  renderer.set_draw_color(color);
  let _ = renderer.fill_rect(rect);
}

fn draw_sprite(sprite: &Sprite, pos: &Position, renderer: &mut Renderer, cam: &Camera) {
  if let Some(ref tex) = sprite.texture {
    let bl = pos + sprite.aabb.center - sprite.aabb.half_size;
    let draw_rect = cam.to_draw_rect(bl, sprite.aabb.half_size * 2.);

    let _ = renderer.copy(
      tex,
      Some(sprite.source_rect.to_sdl_rect()),
      Some(draw_rect)
    );
  }
}

fn draw_physics(position: &Position, collision: &Collision, on_ground: bool, renderer: &mut Renderer, cam: &Camera) {
  let draw_color = if on_ground {
    Color::RGBA(0, 0, 255, 255)
  } else {
    Color::RGBA(255, 0, 0,  255)
  };
  draw_rect(renderer, cam, collision.offset(*position).bottom_left(), collision.half_size * 2., draw_color);
}

fn draw_static(position: &Position, collision: &Collision, renderer: &mut Renderer, cam: &Camera, is_collided: bool) {
  let draw_color = if is_collided { Color::RGBA(0, 255, 255, 255) } else { Color::RGBA(255, 255, 0, 255) };
  draw_rect(renderer, cam, collision.offset(*position).bottom_left(), collision.half_size * 2., draw_color);
}



pub fn render_system(world: &World, renderer: &mut Renderer) {
  if let Some(ref camera) = world.cameras.get(&world.current_camera) {
    //TODO render needs drawing order (z coord?)
    for id in &world.entities {
      if let (Some(ref sprite), Some(ref pos)) =
             (world.sprites.get(&id), world.positions.get(&id))
      {
        draw_sprite(sprite, pos, renderer, camera);
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
    }
  }
}
