extern crate sdl2;
use self::sdl2::keyboard::Keycode;
use self::sdl2::render::Renderer;

use std::path::Path;
use std::time;

use common::{InputState, Vec2};
use components::{
  PlayerAction,
  CameraAction,
  Velocity,
  World,
  DrawObstacleTool,
};
use camera::Camera;
use render;
use physics::simulation_systems;
use editor::{obstacle_tool_input};


pub fn create_world(renderer: &mut Renderer, screen_size: Vec2) -> World {
  let mut world = World::new();
  world.current_player = world.new_player();
  // let tm_id = world.new_tilemap();
  // world.current_tilemap = tm_id;


  let level_size = Vec2::new(120., 30.);
  let bg_id = world.new_background(renderer, level_size / 2., level_size);

  world.new_static_obstacle(Vec2::new(level_size.x / 2., 0.), Vec2::new(level_size.x, 4.));
  world.new_static_obstacle(Vec2::new(16., 6.), Vec2::new(12., 4.));
  world.new_static_obstacle(Vec2::new(28., 8.), Vec2::new(4., 4.));

  world.current_camera = world.new_camera(level_size.y, Vec2::new(0., level_size.y / 2.), screen_size);

  let obstool_id = world.new_entity();
  world.obstacle_tools.insert(obstool_id, DrawObstacleTool::new());

  println!("World created, player {}, camera {}, background {}",
    world.current_player, world.current_camera, bg_id);
  world
}

pub fn player_input_controller(input: &InputState, actions: &mut Vec<PlayerAction>) {
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
  // I can't create or delete entities while iterating on them.
  // TODO Is there a more generic way that I can queue up entity creation?
  let mut create_statics = Vec::new();

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

    if let Some(camera) = world.cameras.get(&world.current_camera) {
      // Systems that need the camera (screen-space tools)
      if let Some(ref mut obstool) = world.obstacle_tools.get_mut(&id) {
        obstacle_tool_input(input, obstool, camera, &mut create_statics);
      }
    }
  }

  for bbox in &create_statics {
    world.new_static_obstacle(bbox.center, bbox.half_size * 2.);
  }

  let remainder_dt = simulation_systems(world, dt);
  render::render_system(world, renderer);

  // return unused time, to be passed through next frame
  remainder_dt
}
