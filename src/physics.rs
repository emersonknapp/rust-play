use std::time;
use std::collections::HashSet;

use common::{Vec2, Vec2u};
use components::{Velocity, Position, World, Collision};
use tilemap::Tilemap;

const GRAVITY: f64 = 500.;

fn camera_follow(camera_pos: &mut Position, following_pos: &Position) {
  camera_pos.x = following_pos.x;
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

pub fn simulation_systems(w: &mut World, dt: time::Duration) -> time::Duration {
  let sim_dt = time::Duration::from_millis(10);
  let sim_dt_secs = sim_dt.as_secs() as f64 + (sim_dt.subsec_nanos() as f64 / 1000000000.);
  let mut dt_accum = dt;


  let mut statics_collisions: HashSet<usize> = HashSet::new();

  //PROPOSAL
  //1. physically update everything (new Position & Velocity, need to keep old)
  //2. detect collisions
  //3. back out collisions (which will affect Velocity based on the original going in)
  for id in &w.entities {
    if let Some((ppos, pcol, _)) = w.get_dynamic_entity(*id) {
      for id in &w.entities {
        if let Some((spos, scol)) = w.get_static_entity(*id) {
          if pcol.offset(*ppos).intersects(&scol.offset(*spos)) {
            statics_collisions.insert(*id);
          }
        }
      }
    }
  }

  w.statics_collisions = statics_collisions;
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
