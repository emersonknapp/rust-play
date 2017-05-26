extern crate sdl2;
mod physics;
mod camera;
mod common;
mod entity;
mod render;

use std::collections::HashSet;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use common::{Vec2, AABB};
use physics::{MovingObject};
use camera::{Camera};
use render::{Renderable, draw, draw_physics};
use entity::Entity;


enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
}

fn player_resolve_actions(player: &mut Entity, actions: &Vec<PlayerAction>) {
  if let Some(ref mut phys) = player.phys {
    phys.speed.x = 0.;
    for a in actions {
      match a {
        &PlayerAction::MoveLeft => phys.speed.x -= 10.,
        &PlayerAction::MoveRight => phys.speed.x += 10.,
        &PlayerAction::Jump => {
          if phys.on_ground {
            phys.speed.y += 150.;
          }
        },
      }
    }
  }
}

fn input_player(keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, do_action: &mut FnMut(PlayerAction)) {
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

// World is a collection of systems. Remember not to let it pass itself anywhere, just relevant bits
struct World {
  player: Entity,
  background: Entity,
  player_pending_actions: Vec<PlayerAction>,
  camera: Camera,
}

impl World {
  fn new(renderer: &mut sdl2::render::Renderer) -> World {
    let level_size = Vec2::new(100., 25.);

    let player = Entity {
      center: Vec2::new(50., 50.),
      rend: None,
      phys: Some(MovingObject {
        pos: Vec2::new(50., 50.),
        speed: Vec2::new(0., 0.),
        bbox: AABB::new(Vec2::new(50., 50.), Vec2::new(1., 1.)),
        on_ground: true,
      }),
    };

    let background = Entity {
      center: Vec2::new(0., 0.),
      rend: Some(Renderable::new(
        renderer,
        "assets/background.png",
        AABB {
          center: level_size / 2.,
          half_size: level_size / 2.,
        }
      )),
      phys: None,
    }
    ;
    World {
      background: background,
      player: player,
      player_pending_actions: Vec::new(),
      camera: Camera {
        fovy: 100.,
        screen_height: 480.,
        pos: Vec2::new(0., 0.)
      }
    }
  }
  fn input(&mut self, keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>) {
    let mut do_action = |a: PlayerAction| {
      self.player_pending_actions.push(a);
    };
    // parse input, (inputs, ?prev_logic_state?) -> actions
    input_player(&keys_down, &pressed, &mut do_action);
  }
  fn update(&mut self, _: f64, dt_seconds: f64) {
    // dispatch actions through reducer, (prev_logic_state, actions) -> next_logic_state
    player_resolve_actions(&mut self.player, &self.player_pending_actions);
    // step the physics simulation, (prev_phys_state, next_logic_state, time) -> next_phys_state
    if let Some(ref mut p) = self.player.phys {
      p.update(dt_seconds);
    }
    // clean up
    self.player_pending_actions.clear();
  }
  fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
    if let Some(ref r) = self.background.rend {
      draw(renderer, r, &self.camera);
    }
    if let Some(ref p) = self.player.phys {
      draw_physics(p, renderer, &self.camera);
    }
  }
}


fn main() {
  // sdl setup
  let sdl_context = sdl2::init().unwrap();
  let _sdl_image_context = sdl2::image::init(sdl2::image::INIT_PNG);
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
  let mut world = World::new(&mut renderer);
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
    renderer.set_draw_color(Color::RGBA(0,0,0,255));
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
