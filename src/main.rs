extern crate sdl2;
#[macro_use]
extern crate bitmask;

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::rect::Rect;

type ID = usize;

bitmask! {
  mask ComponentMask: u32 where flags ComponentType {
    NoComponent = 0,
    Renderable = 1 << 0,
    Physics = 1 << 1,
    Input = 1 << 2,
  }
}

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

struct Input {
}

struct World {
  next: ID,
  masks: Vec<ComponentMask>,
  rendies: Vec<Option<Renderable>>,
  physies: Vec<Option<PhysicsComponent>>,
  inputs: Vec<Option<Input>>,
}

impl World {
  fn new() -> World {
    World {
      next: 0,
      masks: Vec::new(),
      rendies: Vec::new(),
      physies: Vec::new(),
      inputs: Vec::new(),
    }
  }
}

fn draw(renderer: &mut sdl2::render::Renderer, rend: &Renderable, phys: &PhysicsComponent) {
  let mut dest_rect = rend.source_rect;
  dest_rect.center_on(phys.pos);
  renderer.copy_ex(
    &rend.texture, Some(rend.source_rect), Some(dest_rect), 10.0, None, true, false)
    .unwrap();
}

fn character(world: &mut World, renderer: &sdl2::render::Renderer) -> ID {
  let id = world.next;
  let rend = Renderable::new(&renderer, "assets/animate.bmp");
  let phys = PhysicsComponent::new(Point::new(320, 240));
  let mask = ComponentType::Renderable | ComponentType::Physics;
  world.next += 1;
  world.masks.push(mask);
  world.rendies.push(Some(rend));
  world.physies.push(Some(phys));
  world.inputs.push(None);
  id
}

fn render(renderer: &mut sdl2::render::Renderer, ents: &Vec<ID>, world: &World) {
    renderer.clear();
    for e in ents {
      // let renderer_mask = ComponentType::Physics | ComponentType::Renderable;
      match (world.rendies.get(*e), world.physies.get(*e)) {
        (Some(&Some(ref r)), Some(&Some(ref p))) => {
          draw(renderer, &r, &p);
        },
        _ => {}
      }
    }
    renderer.present();
}

fn main() {
  let mut world = World::new();

  // Create Systems
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

  // Create Entities/Components
  let mut enties : Vec<ID> = Vec::new();

  enties.push(character(&mut world, &renderer));

  let mut prev_keys = HashSet::new();

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => break 'running,
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
      break 'running;
    }
    // let released = &prev_keys - &keys;
    prev_keys = keys;

    // Run Systems
    // movement and animations
    for e in &enties {
      match world.physies[*e] {
        Some(ref mut p) => p.update(ticks),
        _ => {}
      }
      match world.rendies[*e] {
        Some(ref mut r) => r.update(ticks),
        _ => {}
      }
    }

    // rendering
    render(&mut renderer, &enties, &world);

    std::thread::sleep(Duration::from_millis(100));
  }
}
