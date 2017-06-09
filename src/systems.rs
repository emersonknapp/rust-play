extern crate sdl2;
use self::sdl2::keyboard::Keycode;
use self::sdl2::mouse::MouseButton;
use self::sdl2::render::Renderer;

use std::path::Path;
use std::time;

use common::{InputState, Vec2, Vec2u};
use components::{
  PlayerAction,
  CameraAction,
  TilemapAction,
  Velocity,
  Position,
  World,
  Collision,
};
use camera::Camera;
use tilemap::Tilemap;
use render;

const GRAVITY: f64 = 500.;

pub fn create_world(renderer: &mut Renderer, screen_size: Vec2) -> World {
  let mut world = World::new();
  world.current_player = world.new_player();
  let tm_id = world.new_tilemap();
  world.current_tilemap = tm_id;

  world.new_static_obstacle(Vec2::new(18., 10.), Vec2::new(5., 5.));
  world.new_static_obstacle(Vec2::new(50., 10.), Vec2::new(5., 5.));

  let level_size;
  let size;
  let pos;
  {
    let ref tiles = world.tilemaps.get(&tm_id).unwrap();
    level_size = Vec2::new(tiles.width as f64, tiles.height as f64);
    size = level_size * tiles.tile_size;
    pos = level_size * tiles.tile_size / 2.;
  }
  let bg_id = world.new_background(renderer, pos, size);

  world.current_camera = world.new_camera(tm_id, screen_size);

  println!("World created, player {}, tilamep {}, camera {}, background {}",
    world.current_player, tm_id, world.current_camera, bg_id);
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
      &PlayerAction::MoveLeft => velocity.x -= 16.,
      &PlayerAction::MoveRight => velocity.x += 16.,
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

fn physics_update(velocity: &mut Velocity, position: &mut Position,
    collision: &Collision, tilemap: &Tilemap, on_ground: &mut bool, dt_seconds: f64)
{
  let dpos = *velocity * dt_seconds;
  let next = *position + dpos;
  velocity.y -= GRAVITY * dt_seconds;

  let mut test = Vec2::new(next.x, position.y);
  // TODO collide with multiple tilemaps?
  let mut test_intersections: Vec<Vec2u> = tilemap.intersects_box(&collision.offset(test));
  if test_intersections.len() > 0 {
    test.x = position.x;
  }
  test.y = next.y;
  test_intersections = tilemap.intersects_box(&collision.offset(test));
  if test_intersections.len() > 0 {
    test.y = position.y;
    if next.y < position.y {
      // landed
      *on_ground = true;
      velocity.y = 0.;
    } else {
      // bonked your head
      velocity.y = velocity.y.min(0.);
    }
  }

  *position = test;
}

fn camera_follow(camera_pos: &mut Position, following_pos: &Position) {
  camera_pos.x = following_pos.x;
}

fn simulation_systems(w: &mut World, dt: time::Duration) -> time::Duration {
  let sim_dt = time::Duration::from_millis(10);
  let sim_dt_secs = sim_dt.as_secs() as f64 + (sim_dt.subsec_nanos() as f64 / 1000000000.);
  let mut dt_accum = dt;


  let pcol = Collision::new(Vec2::new(0., 0.), Vec2::new(10., 10.));
  let mut cols: Vec<usize> = Vec::new();

  //PROPOSAL
  //1. physically update everything (new Position & Velocity, need to keep old)
  //2. detect collisions
  //3. back out collisions (which will affect Velocity based on the original going in)

  for id in &w.entities {
    if let Some((ppos, pcol, _)) = w.get_dynamic_entity(*id) {
      for id in &w.entities {
        if let Some((spos, scol)) = w.get_static_entity(*id) {
          if pcol.offset(*ppos).intersects(&scol.offset(*spos)) {
            cols.push(*id);
          }
        }
      }
    }
  }
  //TODO now that i was capable of writing this loop, how to resolve collisions?


  while dt_accum >= sim_dt {
    for id in &w.entities {
      // run physics
      if let (
        Some(ref mut velocity), Some(ref mut position), Some(ref collision), Some(ref mut on_ground), Some(ref tilemap)
      ) = (
        w.velocities.get_mut(&id), w.positions.get_mut(&id), w.collisions.get(&id), w.groundables.get_mut(&id), w.tilemaps.get(&w.current_tilemap)
      ) {
        physics_update(velocity, position, collision, tilemap, on_ground, sim_dt_secs);
      }

      // update camera follow
      if let (Some(ref mut camera), Some(ref player_pos)) =
             (w.cameras.get_mut(&w.current_camera), w.positions.get(&w.current_player))
      {
        camera_follow(&mut camera.pos, player_pos)
      }
      // TODO reactions to physics collisions?

    }
    dt_accum -= sim_dt;
  }

  dt_accum
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

  // simulation systems
  let remainder_dt = simulation_systems(world, dt);

  // TODO clear all dead entities

  // output systems
  if let Some(ref camera) = world.cameras.get(&world.current_camera) {
    //TODO render needs drawing order (z coord?)
    for id in &world.entities {
      if let (Some(ref sprite), Some(ref pos)) =
             (world.sprites.get(&id), world.positions.get(&id))
      {
        render::draw(sprite, pos, renderer, camera);
      }
    }
    for id in &world.entities {
      if let (Some(ref tilemap),) = (world.tilemaps.get(&id),) {
        render::draw_tilemap_collisions(&tilemap, &Vec::new(), renderer, camera);
      }
    }
    for id in &world.entities {
      if let (Some(ref collision), Some(ref position), None) =
             (world.collisions.get(&id), world.positions.get(&id), world.velocities.get(&id))
      {
        render::draw_statics(position, collision, renderer, camera);
      }
    }
    for id in &world.entities {
      if let (Some(ref collision), Some(ref position), Some(on_ground)) =
             (world.collisions.get(&id), world.positions.get(&id), world.groundables.get(&id))
      {
        render::draw_physics(position, collision, *on_ground, renderer, camera);
      }
    }
  }

  // return unused time, to be passed forward next frame
  remainder_dt
}

// TODO
  // pub fn print_stats(&self) {
  //   if let Some(ref p) = self.player.phys {
  //       println!("pl {} {} {} {}",
  //         self.player.center.x, self.player.center.y, p.speed.x, p.speed.y);
  //   }
  // }
