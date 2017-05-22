#![allow(dead_code)]
extern crate sdl2;
mod physics;

use physics::{vec2, MovingObject, AABB};

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::pixels::Color;

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

// fn draw(renderer: &mut sdl2::render::Renderer, rend: &Renderable) {
//   let mut dest_rect = rend.source_rect;
//   dest_rect.center_on(phys.pos);
//   renderer.copy_ex(
//     &rend.texture, Some(rend.source_rect), Some(dest_rect), 0.0, None, true, false)
//     .unwrap();
// }

// fn render(renderer: &mut sdl2::render::Renderer) {
//     renderer.clear();
//     for e in ents {
//       // let renderer_mask = ComponentType::Physics | ComponentType::Renderable;
//       match (world.rendies.get(*e), world.physies.get(*e)) {
//         (Some(&Some(ref r)), Some(&Some(ref p))) => {
//           draw(renderer, &r, &p);
//         },
//         _ => {}
//       }
//     }
//     renderer.present();
// }
fn aabb_to_rect(a: &AABB) -> Rect {
  // TODO transform into render space
  Rect::new(
    (a.center.x - a.halfSize.x) as i32,
    (a.center.y - a.halfSize.y) as i32,
    (a.halfSize.x * 2.) as u32,
    (a.halfSize.y * 2.) as u32,
  )
}

enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
}
struct Player {

}

fn player_update(actions: &Vec<PlayerAction>, player: &mut MovingObject) {
  player.speed.x = 0.;
  for a in actions {
    match a {
      &PlayerAction::MoveLeft => player.speed.x -= 100.,
      &PlayerAction::MoveRight => player.speed.x += 100.,
      &PlayerAction::Jump => {
        if player.onGround {
          player.speed.y += 100.;
        }
      },
    }
  }
}

fn player_input(keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, do_action: &mut FnMut(PlayerAction) ) {
  if pressed.contains(&Keycode::Space) {
    do_action(PlayerAction::Jump);
  }
  if keys_down.contains(&Keycode::Left) {
    do_action(PlayerAction::MoveLeft);
  }
  if keys_down.contains(&Keycode::Right) {
    do_action(PlayerAction::MoveRight);
  }
}

struct World {
  player: MovingObject,
  pending_actions: Vec<PlayerAction>,
}

impl World {
  fn new() -> World {
    let player = MovingObject {
      pos: vec2::new(10., 10.),
      speed: vec2::new(0., 0.),
      bbox: AABB::new(vec2::new(10., 10.), vec2::new(20., 20.)),
      onGround: true,
    };
    World {
      player: player,
      pending_actions: Vec::new(),
    }
  }
  fn input(&mut self, keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>) {
    let mut do_action = |a: PlayerAction| {
      self.pending_actions.push(a);
    };
    player_input(&keys_down, &pressed, &mut do_action);
  }
  fn update(&mut self, _: f64, dt_seconds: f64) {
    // TODO this is confusing, one is action resolution and the other is physics simulation
    player_update(&self.pending_actions, &mut self.player);
    self.player.update(dt_seconds);
    self.pending_actions.clear();
  }
  fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
    // TODO camera (flips/scales + translates from world coords to pixel coords)
    let mut draw_color = Color::RGBA(255, 0, 0, 255);
    if self.player.onGround {
      draw_color = Color::RGBA(0, 0, 255, 255);
    }
    renderer.set_draw_color(draw_color);
    let draw_rect = aabb_to_rect(&self.player.bbox);
    let _ = renderer.fill_rect(draw_rect);
  }
}


fn main() {
  // sdl setup
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem.window("SDL2", 640, 480)
    .position_centered()
    .build()
    .unwrap();
  let mut renderer = window.renderer()
    .accelerated().build().unwrap();

  let mut timer = sdl_context.timer().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut prev_keys = HashSet::new();
  let mut last_ticks = timer.ticks();
  let mut ticks_leftover = 0;

  // game init
  let mut world = World::new();
  let sim_dt = 10;
  let sim_dt_seconds = sim_dt as f64 / 1000.;

  'running: loop {
    let ticks = timer.ticks();
    let time = ticks as f64 / 1000.;
    let dt = ticks - last_ticks;
    last_ticks = ticks;
    ticks_leftover += dt;

    // input
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => break 'running,
        _ => {}
      }
    }
    let keys: HashSet<Keycode> = event_pump.keyboard_state()
      .pressed_scancodes()
      .filter_map(Keycode::from_scancode)
      .collect();
    let pressed = &keys - &prev_keys;
    if keys.contains(&Keycode::Escape) {
      break 'running;
    }
    // let released = &prev_keys - &keys;

    // prepare for drawing
    renderer.set_draw_color(Color::RGBA(0,255,0,255));
    renderer.clear();

    // invoke game logic
    world.input(&keys, &pressed);
    while ticks_leftover >= sim_dt {
      world.update(time, sim_dt_seconds);
      ticks_leftover -= sim_dt
    }
    world.draw(&mut renderer);

    // loop finalizing
    renderer.present();
    prev_keys = keys;
  }
}

// fn entity_main() {
//   // Create Systems
//   let sdl_context = sdl2::init().unwrap();
//   let video_subsystem = sdl_context.video().unwrap();
//   let window = video_subsystem.window("SDL2", 640, 480)
//     .position_centered()
//     .build()
//     .unwrap();
//   let mut renderer = window.renderer()
//     .accelerated().build().unwrap();
//
//   renderer.set_draw_color(sdl2::pixels::Color::RGBA(0,255,0,255));
//
//   let mut timer = sdl_context.timer().unwrap();
//   let mut event_pump = sdl_context.event_pump().unwrap();
//
//   // Create Entities/Components
//   let mut enties : Vec<ID> = Vec::new();
//
//   enties.push(character(&mut world, &renderer));
//
//   let mut prev_keys = HashSet::new();
//
//   'running: loop {
//     for event in event_pump.poll_iter() {
//       match event {
//         Event::Quit {..} => break 'running,
//         _ => {}
//       }
//     }
//     let ticks = timer.ticks();
//
//     // Input
//     let keys = event_pump.keyboard_state()
//       .pressed_scancodes()
//       .filter_map(Keycode::from_scancode)
//       .collect();
//     let pressed = &keys - &prev_keys;
//     if pressed.contains(&Keycode::Escape) {
//       break 'running;
//     }
//     // let released = &prev_keys - &keys;
//     prev_keys = keys;
//
//     // Run Systems
//     // movement and animations
//     for e in &enties {
//       match world.physies[*e] {
//         Some(ref mut p) => p.update(ticks),
//         _ => {}
//       }
//       match world.rendies[*e] {
//         Some(ref mut r) => r.update(ticks),
//         _ => {}
//       }
//     }
//
//     // rendering
//     render(&mut renderer, &enties, &world);
//
//     std::thread::sleep(Duration::from_millis(100));
//   }
// }
