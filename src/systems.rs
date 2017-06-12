extern crate sdl2;
use self::sdl2::keyboard::Keycode;
use self::sdl2::mouse::MouseButton;
use self::sdl2::render::Renderer;

use std::path::Path;
use std::time;

use common::{InputState, Vec2};
use components::{
  PlayerAction,
  CameraAction,
  TilemapAction,
  Velocity,
  World,
};
use camera::Camera;
use tilemap::Tilemap;
use render;
use physics::simulation_systems;


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
  if input.key_pressed(&Keycode::W) {
    actions.push(CameraAction::MoveUp);
  }
  if input.key_pressed(&Keycode::A) {
    actions.push(CameraAction::MoveLeft);
  }
  if input.key_pressed(&Keycode::S) {
    actions.push(CameraAction::MoveDown);
  }
  if input.key_pressed(&Keycode::D) {
    actions.push(CameraAction::MoveRight);
  }
  if input.key_pressed(&Keycode::Q) {
    actions.push(CameraAction::ZoomOut);
  }
  if input.key_pressed(&Keycode::E) {
    actions.push(CameraAction::ZoomIn);
  }
}

fn tilemap_input_controller(input: &InputState, actions: &mut Vec<TilemapAction>) {
    if input.mouse_pressed(MouseButton::Left) {
      actions.push(TilemapAction::ToggleTileCollision(input.mouse.x(), input.mouse.y()));
    }
    if input.key_down(&Keycode::LCtrl) && input.key_down(&Keycode::S) {
      actions.push(TilemapAction::Save);
    }
}

fn player_update(actions: &Vec<PlayerAction>, velocity: &mut Velocity, on_ground: &mut bool) {
  velocity.x = 0.;
  for a in actions {
    match a {
      &PlayerAction::MoveLeft => velocity.x -= 24.,
      &PlayerAction::MoveRight => velocity.x += 24.,
      &PlayerAction::Jump => {
        if *on_ground {
          velocity.y += 100.;
          *on_ground = false;
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

fn tilemap_update(actions: &Vec<TilemapAction>, camera: &Camera, tilemap: &mut Tilemap) {
  for a in actions {
    match a {
      &TilemapAction::ToggleTileCollision(screen_x, screen_y) => {
        let world_coord = camera.screen2world(screen_x, screen_y);
        if let Some((x, y)) = tilemap.tile_for(world_coord) {
          tilemap.collisions[(y, x)] = !tilemap.collisions[(y, x)]
        }
      },
      &TilemapAction::Save => {
        let _ = tilemap.save(Path::new("assets/modified_level.lv"));
      }
    }
  }
}


pub fn run_systems(world: &mut World, input: &InputState, renderer: &mut Renderer, dt: time::Duration) -> time::Duration {
  for id in &world.entities {
    // input systems
    if let Some(ref mut actions) = world.player_actions.get_mut(&id) {
      player_input_controller(input, actions);
    }
    if let Some(ref mut actions) = world.camera_actions.get_mut(&id) {
      camera_input_controller(input, actions);
    }
    // TODO tilemap editor tool is its own entity
    if let Some(ref mut actions) = world.tilemap_actions.get_mut(&id) {
      tilemap_input_controller(input, actions);
    }

    // logic systems
    // NOTE: use all pending actions in this block, they will be cleared
    if let (
      Some(ref actions), Some(ref mut velocity), Some(ref mut on_ground)
    ) = (
      world.player_actions.get(&id), world.velocities.get_mut(&id), world.groundables.get_mut(&id)
    ) {
      player_update(actions, velocity, on_ground);
    }

    if let (
      Some(ref actions), Some(ref mut camera)
    ) = (
      world.camera_actions.get(&id), world.cameras.get_mut(&id)
    ) {
      camera_update(actions, camera);
    }

    if let (
      Some(ref actions), Some(ref mut tilemap), Some(ref camera)
    ) = (
      world.tilemap_actions.get(&id), world.tilemaps.get_mut(&id), world.cameras.get(&world.current_camera),
    ) {
      tilemap_update(actions, camera, tilemap);
    }

    // Clear up all pending actions, now that they have been resolved
    for (_, actions) in &mut world.player_actions {
      actions.clear();
    }
    for (_, actions) in &mut world.camera_actions {
      actions.clear();
    }
    for (_, actions) in &mut world.tilemap_actions {
      actions.clear();
    }
  }

  let remainder_dt = simulation_systems(world, dt);

  // TODO clear all dead entities

  render::render_system(world, renderer);

  // return unused time, to be passed forward next frame
  remainder_dt
}
