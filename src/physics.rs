use std::time;
use std::collections::{HashSet, HashMap};

use common::{Vec2, Vec2u};
use components::{Velocity, Position, World, Collision};
use tilemap::Tilemap;

const GRAVITY: f64 = 500.;

fn camera_follow(camera_pos: &mut Position, following_pos: &Position) {
  camera_pos.x = following_pos.x;
}

fn movement_update(position: &Position, velocity: &Velocity, dt_seconds: f64) -> (Position, Velocity) {
  let dpos = *velocity * dt_seconds;
  let next = *position + dpos;
  let next_vel = velocity - Vec2::new(0., GRAVITY * dt_seconds);
  (next, next_vel)
}

pub fn physics_step(w: &mut World, dt_seconds: f64, statics_collisions: &mut HashSet<usize>) {
  // move everything first
  // TODO: don't resolve tilemap collisions here, do it in collision detection & resolution phase
  let mut updates: Vec<(usize, Position, Velocity, bool)> = Vec::new();

  for id in &w.entities {
    if let Some((pos, _, vel)) = w.get_dynamic_entity(*id) {
      let (next_pos, next_vel) = movement_update(pos, vel, dt_seconds);
      updates.push((*id, next_pos, next_vel, false));
    }
  }

  /* tilemap collision
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
  */

  // detect statics collisions
  for id in &w.entities {
    if let Some((spos, scol)) = w.get_static_entity(*id) {
      for &(uid, ppos, _, _) in &updates {
        if let Some(pcol) = w.collisions.get(&uid) {
          if pcol.offset(ppos).intersects(&scol.offset(*spos)) {
            statics_collisions.insert(*id);
          }
        }
      }
    }
  }

  // resolve collisions
  // don't allow fall below 0 (for testing)
  for idx in 0..updates.len() {
    if updates[idx].1.y <= 0. {
      updates[idx].1.y = 0.;
      updates[idx].2.y = 0.;
      updates[idx].3 = true;
    }
  }

  // finalize
  for (id, pos, vel, ground) in updates {
    w.positions.insert(id, pos);
    w.velocities.insert(id, vel);
    if let Some(_) = w.groundables.get(&id) {
      w.groundables.insert(id, ground);
    }
  }
}

pub fn simulation_systems(w: &mut World, dt: time::Duration) -> time::Duration {
  let sim_dt = time::Duration::from_millis(10);
  let sim_dt_secs = sim_dt.as_secs() as f64 + (sim_dt.subsec_nanos() as f64 / 1000000000.);
  let mut dt_accum = dt;


  let mut statics_collisions: HashSet<usize> = HashSet::new();


  while dt_accum >= sim_dt {
    physics_step(w, sim_dt_secs, &mut statics_collisions);
    dt_accum -= sim_dt;
  }

  // update camera follow
  if let (Some(ref mut camera), Some(ref player_pos)) =
         (w.cameras.get_mut(&w.current_camera), w.positions.get(&w.current_player))
  {
    camera_follow(&mut camera.pos, player_pos)
  }
  // TODO reactions (damage, animation, sound) to physics collisions?

  w.statics_collisions = statics_collisions;

  dt_accum
}
