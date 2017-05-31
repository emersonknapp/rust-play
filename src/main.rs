extern crate sdl2;
mod physics;
mod camera;
mod common;
mod entity;
mod render;
mod tilemap;

use std::collections::HashSet;
use std::path::Path;
use std::{time, thread};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use common::{Vec2, Vec2u, AABB};
use physics::{MovingObject};
use camera::{Camera};
use render::{Renderable, draw, draw_physics, draw_tilemap_collisions};
use entity::Entity;
use tilemap::{Tilemap};


enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
}

enum CameraAction {
  MoveLeft,
  MoveRight,
  MoveUp,
  MoveDown,
}

fn player_resolve_actions(player: &mut Entity, actions: &Vec<PlayerAction>) {
  if let Some(ref mut phys) = player.phys {
    phys.speed.x = 0.;
    for a in actions {
      match a {
        &PlayerAction::MoveLeft => phys.speed.x -= 16.,
        &PlayerAction::MoveRight => phys.speed.x += 16.,
        &PlayerAction::Jump => {
          if phys.on_ground {
            phys.speed.y += 100.;
            phys.on_ground = false;
          }
        },
      }
    }
  }
}

fn input_player(keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, released: &HashSet<Keycode>, do_action: &mut FnMut(PlayerAction)) {
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

fn camera_resolve_actions(camera: &mut Camera, actions: &Vec<CameraAction>) {
  for a in actions {
    match a {
      &CameraAction::MoveLeft => camera.pos.x -= 2.,
      &CameraAction::MoveRight => camera.pos.x += 2.,
      &CameraAction::MoveUp => camera.pos.y -= 2.,
      &CameraAction::MoveDown => camera.pos.y += 2.,
    }
  }
}
fn input_camera(_: &HashSet<Keycode>, pressed: &HashSet<Keycode>, do_action: &mut FnMut(CameraAction)) {
  if pressed.contains(&Keycode::W) {
    do_action(CameraAction::MoveUp);
  }
  if pressed.contains(&Keycode::A) {
    do_action(CameraAction::MoveLeft);
  }
  if pressed.contains(&Keycode::S) {
    do_action(CameraAction::MoveDown);
  }
  if pressed.contains(&Keycode::D) {
    do_action(CameraAction::MoveRight);
  }
}

// World is a collection of systems. Remember not to let it pass itself anywhere, just relevant bits
struct World {
  player: Entity,
  background: Entity,
  tilemap: Tilemap,
  player_pending_actions: Vec<PlayerAction>,
  camera_pending_actions: Vec<CameraAction>,
  tilemap_intersectons: Vec<Vec2u>,
  camera: Camera,
}

impl World {
  fn new(renderer: &mut sdl2::render::Renderer, screen_size: Vec2) -> World {
    let tiles = Tilemap::from_file(Path::new("assets/level1.lv")).unwrap();
    let level_size = Vec2::new(tiles.width as f64, tiles.height as f64);


    let player = Entity {
      center: Vec2::new(8., 20.),
      rend: None,
      phys: Some(MovingObject {
        speed: Vec2::new(0., 0.),
        aabb: AABB::new(Vec2::new(0., 0.), Vec2::new(0.95, 0.95)),
        on_ground: true,
      }),
    };

    let background = Entity {
      center: Vec2::new(0., 0.),
      rend: Some(Renderable::new(
        renderer,
        "assets/background.png",
        AABB {
          center: level_size * tiles.tile_size / 2.,
          half_size: level_size * tiles.tile_size / 2.,
        }
      )),
      phys: None,
    };

    World {
      background: background,
      player: player,
      player_pending_actions: Vec::new(),
      camera_pending_actions: Vec::new(),
      camera: Camera {
        fovy: level_size.y * tiles.tile_size,
        screen_height: screen_size.y,
        pos: Vec2::new(0., 0.)
      },
      tilemap: tiles,
      tilemap_intersectons: Vec::new(),
    }
  }
  fn input(&mut self, keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, released: &HashSet<Keycode>) {
    {
      let mut do_action = |a: PlayerAction| {
        self.player_pending_actions.push(a);
      };
      // parse input, (inputs, ?prev_logic_state?) -> actions
      input_player(&keys_down, &pressed, &released, &mut do_action);
    }
    {
      let mut do_cam_action = |a: CameraAction| {
        self.camera_pending_actions.push(a);
      };
      input_camera(&keys_down, &pressed, &mut do_cam_action);
    }
    // dispatch actions through reducer, (prev_logic_state, actions) -> next_logic_state
    player_resolve_actions(&mut self.player, &self.player_pending_actions);
    camera_resolve_actions(&mut self.camera, &self.camera_pending_actions);
    // clean up
    self.player_pending_actions.clear();
    self.camera_pending_actions.clear();
  }
  fn update(&mut self, _: f64, dt_seconds: f64) {
    // TODO: move out of world to phys
    // TODO: the faster you approach an obstacle, the farther from it you stop because of update distance & backing out
    // step the physics simulation, (prev_phys_state, next_logic_state, time) -> next_phys_state
    if let Some(ref mut p) = self.player.phys {
      let next_center = p.update(self.player.center, dt_seconds);
      let mut test_center = Vec2::new(next_center.x, self.player.center.y);
      let mut tis = self.tilemap.intersects_box(&p.aabb.offset(test_center));
      if tis.len() > 0 {
        test_center.x = self.player.center.x;
      }
      test_center.y = next_center.y;
      tis = self.tilemap.intersects_box(&p.aabb.offset(test_center));
      if tis.len() > 0 {
        test_center.y = self.player.center.y;
        if next_center.y < self.player.center.y {
          p.on_ground = true;
          p.speed.y = 0.;
        } else {
          p.speed.y = p.speed.y.min(0.);
        }
      }
      self.player.center = test_center;
    }
  }
  fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
    if let Some(_) = self.background.rend {
      draw(&self.background, renderer, &self.camera);
    }
    if true {
      draw_tilemap_collisions(&self.tilemap, &self.tilemap_intersectons, renderer, &self.camera);
    }
    if let Some(_) = self.player.phys {
      draw_physics(&self.player, renderer, &self.camera);
    }
  }
}

// fn to_millis(dt: &time::Duration) -> f64 {
//   dt.as_secs() as f64 * 1000. + (dt.subsec_nanos() as f64 / 1000000.)
// }

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
  renderer.set_blend_mode(sdl2::render::BlendMode::Blend);

  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut prev_keys = HashSet::new();

  // game init
  let mut world = World::new(&mut renderer, Vec2::new(640., 480.));

  let sim_dt = time::Duration::from_millis(10);
  let sim_dt_secs = sim_dt.as_secs() as f64 + (sim_dt.subsec_nanos() as f64 / 1000000000.);
  let target_frame_time = time::Duration::from_millis(16);
  let mut last_time = time::Instant::now();
  let mut dt_accum = time::Duration::new(0, 0);

  // debug stuff
  let mut frame_counter = 0;
  let mut phys_counter = 0;
  let mut frame_counter_accumulator = time::Duration::new(0, 0);

  thread::sleep(target_frame_time);
  'running: loop {
    let dt = last_time.elapsed();
    last_time = time::Instant::now();
    assert!(dt >= target_frame_time);
    dt_accum += dt;

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
    let released = &prev_keys - &keys;

    // prepare for drawing
    renderer.set_draw_color(Color::RGBA(0,0,0,255));
    renderer.clear();

    // invoke game logic
    world.input(&keys, &pressed, &released);
    while dt_accum >= sim_dt {
      phys_counter += 1;
      world.update(0., sim_dt_secs);
      dt_accum -= sim_dt;
    }
    world.draw(&mut renderer);

    // loop finalizing
    renderer.present();
    prev_keys = keys;

    // Debug output
    frame_counter += 1;
    frame_counter_accumulator += dt;
    if frame_counter_accumulator.as_secs() > 1 {
      println!("{} {}", frame_counter, phys_counter);
      if let Some(ref p) = world.player.phys {
        println!("pl {:?} {} {} {}", world.player.center.x, world.player.center.y, p.speed.x, p.speed.y);
      }
      frame_counter_accumulator -= time::Duration::from_secs(1);
      frame_counter = 0;
      phys_counter = 0;
    }

    // Sleep until next frame
    if let Some(sleep_duration) = target_frame_time.checked_sub(last_time.elapsed()) {
      thread::sleep(sleep_duration);
    }
  }
}
