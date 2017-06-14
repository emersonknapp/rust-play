extern crate sdl2;
extern crate serde;
extern crate serde_json;

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use common::{Vec2, AABB};
use render::Sprite;
use camera::Camera;

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CameraAction {
  MoveLeft,
  MoveRight,
  MoveUp,
  MoveDown,
  ZoomOut,
  ZoomIn,
}

// Components
pub type Position = Vec2;

pub type Collision = AABB;

pub type Velocity = Vec2;

type Groundable = bool;

pub type PlayerActions = Vec<PlayerAction>;
pub type CameraActions = Vec<CameraAction>;


type ID = usize;

#[derive(Serialize, Deserialize, Debug)]
pub struct World {
  pub positions: HashMap<ID, Position>,

  pub sprites: HashMap<ID, Sprite>,
  pub collisions: HashMap<ID, Collision>,
  pub velocities: HashMap<ID, Velocity>,
  pub groundables: HashMap<ID, Groundable>,
  pub cameras: HashMap<ID, Camera>,

  pub entities: HashSet<ID>,
  next: ID,

  pub current_camera: ID,
  pub current_player: ID,
  pub current_tilemap: ID,

  // It may seem like these are ephemeral,
  // but their presence indicates a thing that can have actions
  #[serde(default)]
  pub player_actions: HashMap<ID, PlayerActions>,
  #[serde(default)]
  pub camera_actions: HashMap<ID, CameraActions>,

  // This is truly ephemeral state
  #[serde(skip)]
  pub statics_collisions: HashSet<ID>,
}

impl World {
  pub fn new() -> World {
    World {
      positions: HashMap::new(),
      sprites: HashMap::new(),
      collisions: HashMap::new(),
      velocities: HashMap::new(),
      groundables: HashMap::new(),
      cameras: HashMap::new(),

      player_actions: HashMap::new(),
      camera_actions: HashMap::new(),

      entities: HashSet::new(),
      next: 1,

      current_camera: 0,
      current_player: 0,
      current_tilemap: 0,

      statics_collisions: HashSet::new(),
    }
  }
  pub fn from_file(path: &Path, renderer: &sdl2::render::Renderer) -> Result<World, io::Error> {
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut world: World = serde_json::from_str(&contents).unwrap();
    // Post-deserialize initialization
    for (_, ref mut sprite) in world.sprites.iter_mut() {
      sprite.reload_assets(renderer);
    }

    println!("deserialized = {:?}", world);
    Ok(world)
  }

  pub fn new_entity(&mut self) -> ID {
    let id = self.next;
    self.entities.insert(id);
    self.next += 1;
    id
  }

  // fn delete_entity(&mut self, id: ID) {
  //   self.positions.remove(&id);
  //   self.sprites.remove(&id);
  //   self.collisions.remove(&id);
  //   self.velocities.remove(&id);
  //   self.groundables.remove(&id);
  //   self.tilemaps.remove(&id);
  //   self.cameras.remove(&id);
  //
  //   self.player_actions.remove(&id);
  //   self.camera_actions.remove(&id);
  //   self.tilemap_actions.remove(&id);
  //
  //   self.entities.remove(&id);
  // }

  pub fn new_player(&mut self) -> ID {
    let id = self.new_entity();
    self.positions.insert(id, Position::new(8., 4.));
    // self.sprites.insert(id, Sprite {
    //
    // };
    self.collisions.insert(id, Collision::new(
      Vec2::new(0., 0.), Vec2::new(1., 2.))
    );
    self.velocities.insert(id, Velocity::new(0., 0.));
    self.groundables.insert(id, false);
    self.player_actions.insert(id, Vec::new());
    id
  }

  pub fn new_camera(&mut self, fovy: f64, pos: Vec2, screen_size: Vec2) -> ID {
    let id = self.new_entity();
    self.cameras.insert(id, Camera {
      fovy: fovy,
      screen_height: screen_size.y,
      ratio: screen_size.x / screen_size.y,
      pos: pos
    });
    self.camera_actions.insert(id, Vec::new());
    id
  }

  pub fn new_background(&mut self, renderer: &mut sdl2::render::Renderer, center: Vec2, size: Vec2) -> ID {
    let id = self.new_entity();
    self.positions.insert(id, Position::new(0., 0.));
    self.sprites.insert(id, Sprite::new(
      renderer,
      "assets/background.png",
      AABB::new(center, size / 2.),
    ));
    id
  }

  pub fn new_static_obstacle(&mut self, center: Vec2, size: Vec2) -> ID {
    let id = self.new_entity();
    self.positions.insert(id, center);
    self.collisions.insert(id, Collision::new(
      Vec2::new(0., 0.), size / 2.,
    ));
    // self.sprites.insert(id, Sprite::new()
    id
  }

  pub fn get_moving_entity(&self, id: ID) -> Option<(&Position, &Velocity)> {
    match (self.positions.get(&id), self.velocities.get(&id)) {
      (Some(p), Some(v)) => Some((p, v)),
      _ => None
    }
  }

  pub fn get_collider_entity(&self, id: ID) -> Option<(&Position, &Collision)> {
    match (self.positions.get(&id), self.collisions.get(&id)) {
      (Some(p), Some(c)) => Some((p, c)),
      _ => None
    }
  }

  pub fn save(&self) -> Result<String, io::Error> {
    let serialized = serde_json::to_string(&self).unwrap();
    println!("serialized = {}", serialized);

    let mut file = File::create(Path::new("assets/w0.air"))?;
    let _ = file.write_all(serialized.as_bytes());
    Ok("good job".to_owned())
  }
}
