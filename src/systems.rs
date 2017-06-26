extern crate sdl2;
use self::sdl2::keyboard::Keycode;
pub use self::sdl2::render::Renderer;

use std::time;
use std::path::Path;

use common::{InputState, Vec2};
use components::{
  PlayerAction,
  CameraAction,
  Velocity,
  World,
};
use camera::Camera;
use render;
use physics::simulation_systems;


pub fn create_world(renderer: &mut Renderer, screen_size: Vec2) -> World {
  // Try loading, fall back to default creation on failure
  let mut world;
  if let Ok(w) = World::from_file(&Path::new("assets/w0.air"), renderer) {
    world = w;
  } else {
    world = World::new();
  }

  world.current_player = world.new_player();

  world.new_mover_block(Vec2::new(0., 10.), Vec2::new(20., 10.), 4.);

  println!("World created, player {}, camera {}",
    world.current_player, world.current_camera );
  world
}

fn player_input_controller(input: &InputState, actions: &mut Vec<PlayerAction>) {
  if input.key_pressed(&Keycode::Space) {
    actions.push(PlayerAction::Jump);
  }
  if input.key_down(&Keycode::Left) {
    actions.push(PlayerAction::MoveLeft);
  }
  if input.key_down(&Keycode::Right) {
    actions.push(PlayerAction::MoveRight);
  }
}

fn camera_input_controller(input: &InputState, actions: &mut Vec<CameraAction>) {
  if !input.key_mod.is_empty() {
    return;
  }
  let key_map = vec![
    (Keycode::W, CameraAction::MoveUp),
    (Keycode::A, CameraAction::MoveLeft),
    (Keycode::S, CameraAction::MoveDown),
    (Keycode::D, CameraAction::MoveRight),
    (Keycode::Q, CameraAction::ZoomOut),
    (Keycode::E, CameraAction::ZoomIn),
  ];
  for (key, action) in key_map {
    if input.key_pressed(&key) {
      actions.push(action);
    }
  }
}

fn player_update(actions: &Vec<PlayerAction>, velocity: &mut Velocity, on_ground: &bool) {
  velocity.x = 0.;
  for a in actions {
    match a {
      &PlayerAction::MoveLeft => velocity.x -= 24.,
      &PlayerAction::MoveRight => velocity.x += 24.,
      &PlayerAction::Jump => {
        if *on_ground {
          velocity.y += 100.;
        }
      },
    }
  }
}

fn camera_update(actions: &Vec<CameraAction>, cam: &mut Camera) {
  for a in actions {
    match a {
      &CameraAction::MoveLeft => cam.pos.x -= 2.,
      &CameraAction::MoveRight => cam.pos.x += 2.,
      &CameraAction::MoveUp => cam.pos.y += 2.,
      &CameraAction::MoveDown => cam.pos.y -= 2.,
      &CameraAction::ZoomOut => cam.fovy *= 1.25,
      &CameraAction::ZoomIn => cam.fovy *= 0.8,
    }
  }
}

pub fn run_systems(world: &mut World, input: &InputState, renderer: &mut Renderer, dt: time::Duration) -> time::Duration {
  for id in &world.entities {
    // input & update systems
    if let Some(ref mut actions) = world.player_actions.get_mut(&id) {
      player_input_controller(input, actions);
      if let (Some(ref mut velocity), Some(ref on_ground)) = (world.velocities.get_mut(&id), world.groundables.get(&id)) {
        player_update(actions, velocity, on_ground);
      }
      actions.clear();
    }

    if let Some(ref mut actions) = world.camera_actions.get_mut(&id) {
      camera_input_controller(input, actions);
      if let Some(ref mut camera) = world.cameras.get_mut(&id) {
        camera_update(actions, camera);
      }
      actions.clear();
    }
  }

  let remainder_dt = simulation_systems(world, dt);
  render::render_system(world, renderer);

  // return unused time, to be passed through next frame
  remainder_dt
}
