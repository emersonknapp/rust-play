extern crate sdl2;

use self::sdl2::render::Renderer;
use self::sdl2::keyboard::Keycode;

use std::collections::HashSet;
use std::path::Path;

use camera::{Camera};
use common::{Vec2, Vec2u, AABB};
use entity::Entity;
use physics::{MovingObject};
use render::{Renderable, draw, draw_physics, draw_tilemap_collisions};
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
  ZoomOut,
  ZoomIn,
}

enum ProgramMode {
  Game,
  TilemapEdit,
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
      &CameraAction::ZoomOut => camera.fovy *= 1.25,
      &CameraAction::ZoomIn => camera.fovy *= 0.8,
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
  if pressed.contains(&Keycode::Q) {
    do_action(CameraAction::ZoomOut);
  }
  if pressed.contains(&Keycode::E) {
    do_action(CameraAction::ZoomIn);
  }
}

fn update_camera(dt_seconds: f64, cam: &mut Camera, following: &Entity) {
  cam.pos.x = following.center.x;
}

// World is a collection of systems. Remember not to let it pass itself anywhere, just relevant bits
pub struct World {
  player: Entity,
  background: Entity,
  tilemap: Tilemap,
  player_pending_actions: Vec<PlayerAction>,
  camera_pending_actions: Vec<CameraAction>,
  tilemap_intersectons: Vec<Vec2u>,
  camera: Camera,
}

impl World {
  pub fn new(renderer: &mut Renderer, screen_size: Vec2) -> World {
    let tiles = Tilemap::from_file(Path::new("assets/level1.lv")).unwrap();
    let level_size = Vec2::new(tiles.width as f64, tiles.height as f64);


    let player = Entity {
      center: Vec2::new(8., 20.),
      rend: None,
      phys: Some(MovingObject {
        speed: Vec2::new(0., 0.),
        aabb: AABB::new(Vec2::new(0., 0.), Vec2::new(0.95, 1.45)),
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
        ratio: screen_size.x / screen_size.y,
        pos: Vec2::new(0., tiles.height as f64/ 2. * tiles.tile_size)
      },
      tilemap: tiles,
      tilemap_intersectons: Vec::new(),
    }
  }
  pub fn input(&mut self, keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, released: &HashSet<Keycode>) {
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
  pub fn update(&mut self, _: f64, dt_seconds: f64) {
    // TODO: move out of world to phys
    // TODO: the faster you approach an obstacle, the farther from it you stop because of update distance & backing out,
    // and therefore on landing, the ground "cushions" your fall, because a slower falling speed after landing will bring you closer
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
    update_camera(dt_seconds, &mut self.camera, &self.player);
  }
  pub fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
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
  pub fn print_stats(&self) {
    if let Some(ref p) = self.player.phys {
        println!("pl {} {} {} {}",
          self.player.center.x, self.player.center.y, p.speed.x, p.speed.y);
    }
  }
}
