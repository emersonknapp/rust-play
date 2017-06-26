use std::time;
use std::collections::{HashSet, HashMap};

use common::{Vec2};
use components::{Velocity, Position, World, Collision};
use platforms::mover_blocks;
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

struct UpdateContainer {
  pos: Vec2,
  next_pos: Vec2,
  // vel: Vec2,
  next_vel: Vec2,
}

fn find_collisions(w: &World, mover_id: usize, mover_collision: &Collision, mover_pos: &Position) -> Vec<(usize, Vec2)> {
  let mut collisions = Vec::new();
  for id in &w.entities {
    if *id == mover_id {
      continue;
    }
    if let Some((pos, coll)) = w.get_collider_entity(*id) {
      let mover_abs = mover_collision.offset(*mover_pos);
      let coll_abs = coll.offset(*pos);
      if let Some(isect) = mover_abs.intersect(&coll_abs) {
        collisions.push((*id, isect));
      }
    }
  }
  collisions
}

fn physics_step(w: &mut World, dt_seconds: f64, debug_collisions: &mut HashSet<usize>) {
  // TODO physics (jump height, fall distance) seems a little inconsistent, do some tests
  let mut move_updates: HashMap<usize, UpdateContainer> = HashMap::new();
  let mut ground_updates: HashMap<usize, bool> = HashMap::new();

  // calculate potential next state
  for id in &w.entities {
    if let Some((pos, vel)) = w.get_moving_entity(*id) {
      let (next_pos, next_vel) = movement_update(pos, vel, dt_seconds);
      move_updates.insert(*id, UpdateContainer {
        pos: *pos,
        next_pos: next_pos,
        // vel: *vel,
        next_vel: next_vel,
      });
      ground_updates.insert(*id, false);
    }
  }

  // detect && resolve collisions
  // TODO this does double the checks, could resolve pairs simultaneously for cheaper
  // TODO how does this do on moving-to-moving collisions?
  for (mover_id, ref mut update) in &mut move_updates {
    // Don't need to check collisions if the mover is not collidable
    if let Some(mover_collision) = w.collisions.get(&mover_id) {
      // First update and test X movement
      let mut test_pos = Vec2::new(update.next_pos.x, update.pos.y);
      let found_collisions = find_collisions(w, *mover_id, mover_collision, &test_pos);
      let x_direction = (update.next_pos.x - update.pos.x).signum();
      let mut backout: f64 = 0.;
      if found_collisions.len() > 0 {
        found_collisions.iter().map(|&(id, overlap)| {
          debug_collisions.insert(id);
          if overlap.x.signum() != x_direction && overlap.x.abs() > backout.abs() {
            backout = overlap.x;
          }
        }).count();
        test_pos.x += backout;
      }

      // Test and update Y movement
      test_pos.y = update.next_pos.y;
      let found_collisions = find_collisions(w, *mover_id, mover_collision, &test_pos);
      if found_collisions.len() > 0 {
        let y_direction = (update.next_pos.y - update.pos.y).signum();
        let mut backout: f64 = 0.;
        found_collisions.iter().map(|&(id, overlap)| {
          if overlap.y.signum() != y_direction && overlap.y.abs() > backout.abs() {
            backout = overlap.y;
          }
          debug_collisions.insert(id);
        }).count();
        test_pos.y += backout;

        // Additional side effects of a vertical collision
        if update.next_pos.y < update.pos.y {
          // landed
          ground_updates.insert(*mover_id, true);
          update.next_vel.y = 0.;
        } else {
          // bonked your head
          update.next_vel.y = update.next_vel.y.min(0.);
        }
      }
      update.next_pos = test_pos;
    }
  }

  // don't allow fall below 0 (for testing)
  for (id, ref mut update) in move_updates.iter_mut() {
    if update.next_pos.y <= 0. {
      update.next_pos.y = 0.;
      update.next_vel.y = 0.;
      ground_updates.insert(*id, true);
    }
  }

  // finalize
  for (id, update) in move_updates {
    w.positions.insert(id, update.next_pos);
    w.velocities.insert(id, update.next_vel);
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
    mover_blocks(w, sim_dt_secs);
    dt_accum -= sim_dt;
  }

  // update camera follow TODO have a camera follower component that knows about what to follow
  if let (Some(ref mut camera), Some(ref player_pos)) =
         (w.cameras.get_mut(&w.current_camera), w.positions.get(&w.current_player))
  {
    camera_follow(&mut camera.pos, player_pos)
  }

  // TODO reactions (damage, animation, sound) to physics collisions?

  w.statics_collisions = statics_collisions;

  dt_accum
}
