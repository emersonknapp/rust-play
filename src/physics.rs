use std::time;
use std::collections::{HashSet, HashMap};

use common::{Vec2};
use components::{Velocity, Position, World};
// use tilemap::Tilemap;

const GRAVITY: f64 = 500.;

fn camera_follow(camera_pos: &mut Position, following_pos: &Position) {
  camera_pos.x = following_pos.x;
}

fn movement_update(position: &Position, velocity: &Velocity, dt_seconds: f64) -> (Position, Velocity) {
  let next_vel = velocity - Vec2::new(0., GRAVITY * dt_seconds);
  let dpos = next_vel * dt_seconds;
  let next = *position + dpos;
  (next, next_vel)
}

pub fn physics_step(w: &mut World, dt_seconds: f64, statics_collisions: &mut HashSet<usize>) {
  // move everything first
  // TODO: don't resolve tilemap collisions here, do it in collision detection & resolution phase
  let mut move_updates: HashMap<usize, (Position, Velocity)> = HashMap::new();
  let mut ground_updates: HashMap<usize, bool> = HashMap::new();

  for id in &w.entities {
    if let Some((pos, vel)) = w.get_moving_entity(*id) {
      let (next_pos, next_vel) = movement_update(pos, vel, dt_seconds);
      move_updates.insert(*id, (next_pos, next_vel));
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


  // detect collisions
  // TODO this does double the work necessary by checking each ordering of each collision
  // TODO this does not order axis updates, which makes me stick to walls
  let mut collisions: Vec<(usize, Vec2, Velocity)> = Vec::new();
  for (mover_id, &(mover_pos, mover_vel)) in &move_updates {
    // Don't need to check collisions if the mover is not collidable
    if let Some(mover_collision) = w.collisions.get(&mover_id) {
      for eid in &w.entities {
        // Don't collide with my damn self
        if eid == mover_id {
          continue;
        }

        if let Some((static_pos, static_collision)) = w.get_collider_entity(*eid) {
          let mover_abs = mover_collision.offset(mover_pos);
          let static_abs = static_collision.offset(*static_pos);
          if let Some(intersection) = mover_abs.intersect(&static_abs) {
            statics_collisions.insert(*eid);
            collisions.push((*mover_id, intersection, mover_vel));
          }
        }
      }
    }
  }

  // resolve collisions
  for &(id, overlap, _) in &collisions {
    // TODO don't just undo move, back out a specific amount
    // TODO This is a stupid way to get position, we KNOW it exists
    let def = Vec2::new(0., 0.);
    let corrected_pos: Vec2 = *w.positions.get(&id).unwrap_or(&def);

    // TODO this new velocity is wrong
    if overlap.y > 0. {
      ground_updates.insert(id, true);
    }
    move_updates.insert(id, (corrected_pos, Vec2::new(0., 0.)));
  }

  // don't allow fall below 0 (for testing)
  for (id, val) in move_updates.iter_mut() {
    if val.0.y <= 0. {
      val.0.y = 0.;
      val.1.y = 0.;
      ground_updates.insert(*id, true);
    }
  }

  // finalize
  for (id, (pos, vel)) in move_updates {
    w.positions.insert(id, pos);
    w.velocities.insert(id, vel);
    if let Some(grounded) = ground_updates.get(&id) {
      w.groundables.insert(id, *grounded);
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
