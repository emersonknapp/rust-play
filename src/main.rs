extern crate sdl2;

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;

type ID = usize;

struct Renderable {
  texture: sdl2::render::Texture,
  source_rect: sdl2::rect::Rect,
}

impl Renderable {
  fn new(renderer: &sdl2::render::Renderer, tex_path: &str) -> Renderable {
    let temp_surface = sdl2::surface::Surface::load_bmp(Path::new(tex_path)).unwrap();
    Renderable {
      texture: renderer.create_texture_from_surface(&temp_surface).unwrap(),
      source_rect: Rect::new(0, 0, 128, 82),
    }
  }

  fn update(&mut self, ticks: u32) {
    self.source_rect.set_x((128 * ((ticks / 100) % 6) ) as i32);
  }
}

struct PhysicsComponent {
    pos: Point,
}

impl PhysicsComponent {
  fn new(pos: Point) -> PhysicsComponent {
    PhysicsComponent {
      pos: pos,
    }
  }

  fn update(&mut self, ticks: u32) {

  }
}

struct Entity {
  rend: Option<ID>,
  phys: Option<ID>,
}

fn draw(renderer: &mut sdl2::render::Renderer, rend: &Renderable, phys: &PhysicsComponent) {
  let mut dest_rect = rend.source_rect;
  dest_rect.center_on(phys.pos);
  renderer.copy_ex(
    &rend.texture, Some(rend.source_rect), Some(dest_rect), 10.0, None, true, false)
    .unwrap();
}


fn main() {

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem.window("SDL2", 640, 480)
    .position_centered()
    .build()
    .unwrap();
  let mut renderer = window.renderer()
    .accelerated().build().unwrap();

  renderer.set_draw_color(sdl2::pixels::Color::RGBA(0,0,0,255));

  let mut timer = sdl_context.timer().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut enties : Vec<Entity> = Vec::new();
  let mut rendies : Vec<Renderable> = Vec::new();
  let mut physies : Vec<PhysicsComponent> = Vec::new();

  let rend0 = Renderable::new(&renderer, "assets/animate.bmp");
  let rend0_id = rendies.len();
  rendies.push(rend0);

  let phys0 = PhysicsComponent::new(Point::new(320, 240));
  let phys0_id = physies.len();
  physies.push(phys0);

  let ent0 = Entity { rend: Some(rend0_id), phys: Some(phys0_id) };
  enties.push(ent0);

  let mut prev_keys = HashSet::new();

  let mut running = true;
  while running {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => {
          running = false;
        },
        _ => {}
      }
    }
    let ticks = timer.ticks();

    // Input
    let keys = event_pump.keyboard_state()
      .pressed_scancodes()
      .filter_map(Keycode::from_scancode)
      .collect();
    let pressed = &keys - &prev_keys;
    if pressed.contains(&Keycode::Escape) {
      running = false;
    }
    let released = &prev_keys - &keys;
    if !released.is_empty() {
      println!("released: {:?}", released);
    }
    prev_keys = keys;

    // Update physics
    for e in &enties {
      match e.phys {
        Some(p) => physies[p].update(ticks),
        _ => {}
      }
    }

    // Update visuals
    for e in &enties {
      match e.rend {
        Some(r) => rendies[r].update(ticks),
        _ => {}
      }
    }

    // Draw
    renderer.clear();
    for e in &enties {
      match (e.rend, e.phys) {
        (Some(r), Some(p)) => draw(&mut renderer, &rendies[r], &physies[p]),
        _ => {}
      }
    }
    renderer.present();

    std::thread::sleep(Duration::from_millis(100));
  }
}
