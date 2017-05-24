extern crate sdl2;
mod physics;

use physics::{vec2, MovingObject, AABB};

use std::collections::HashSet;
use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::image::{LoadSurface};

struct Renderable {
  texture: sdl2::render::Texture,
  source_rect: sdl2::rect::Rect,
}

impl Renderable {
  fn new(renderer: &sdl2::render::Renderer, tex_path: &str) -> Renderable {
    let p = Path::new(tex_path);
    let surf = sdl2::surface::Surface::from_file(p).unwrap();
    let texture = renderer.create_texture_from_surface(&surf).unwrap();
    Renderable {
      texture: texture,
      source_rect: Rect::new(0, 0, surf.width(), surf.height()),
    }
  }

  // Does sprite sheet shifting. Need to write a parameterized version w/ source_rect init as well
  // fn update(&mut self, ticks: u32) {
  //   self.source_rect.set_x((128 * ((ticks / 100) % 6) ) as i32);
  // }
}


// fn render(renderer: &mut sdl2::render::Renderer) {
//     renderer.clear();
//     for e in ents {
//       // let renderer_mask = ComponentType::Physics | ComponentType::Renderable;
//       match (world.rendies.get(*e), world.physies.get(*e)) {
//         (Some(&Some(ref r)), Some(&Some(ref p))) => {
//           draw(renderer, &r, &p);
//         },
//         _ => {}
//       }
//     }
//     renderer.present();
// }
fn aabb_to_rect(a: &AABB) -> Rect {
  // TODO transform into render space
  Rect::new(
    (a.center.x - a.halfSize.x) as i32,
    (a.center.y - a.halfSize.y) as i32,
    (a.halfSize.x * 2.) as u32,
    (a.halfSize.y * 2.) as u32,
  )
}

enum PlayerAction {
  MoveLeft,
  MoveRight,
  Jump,
}

fn player_resolve_actions(actions: &Vec<PlayerAction>, player: &mut MovingObject) {
  player.speed.x = 0.;
  for a in actions {
    match a {
      &PlayerAction::MoveLeft => player.speed.x -= 70.,
      &PlayerAction::MoveRight => player.speed.x += 70.,
      &PlayerAction::Jump => {
        if player.onGround {
          player.speed.y += 200.;
        }
      },
    }
  }
}

fn input_player(keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>, do_action: &mut FnMut(PlayerAction)) {
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

fn draw_physics(object: &MovingObject, renderer: &mut sdl2::render::Renderer) {
  let ground_color = Color::RGBA(0, 0, 255, 255);
  let air_color = Color::RGBA(255, 0, 0,  255);
  let draw_color = if object.onGround { ground_color } else { air_color };
  let draw_rect = aabb_to_rect(&object.bbox);
  renderer.set_draw_color(draw_color);
  let _ = renderer.fill_rect(draw_rect);
}

fn draw(renderer: &mut sdl2::render::Renderer, rend: &Renderable) {
  let dest_rect = rend.source_rect;
  // dest_rect.center_on(phys.pos);
  renderer.copy_ex(
    &rend.texture, Some(rend.source_rect), Some(dest_rect), 0.0, None, true, false)
    .unwrap();
}

// World is a collection of systems. Remember not to let it pass itself anywhere, just relevant bits
struct World {
  player_physics: MovingObject,
  player_pending_actions: Vec<PlayerAction>,
  background: Renderable,
}

impl World {
  fn new(renderer: &mut sdl2::render::Renderer) -> World {
    let player = MovingObject {
      pos: vec2::new(10., 10.),
      speed: vec2::new(0., 0.),
      bbox: AABB::new(vec2::new(10., 10.), vec2::new(20., 20.)),
      onGround: true,
    };
    let background = Renderable::new(renderer, "assets/background.png");
    World {
      player_physics: player,
      player_pending_actions: Vec::new(),
      background: background,
    }
  }
  fn input(&mut self, keys_down: &HashSet<Keycode>, pressed: &HashSet<Keycode>) {
    let mut do_action = |a: PlayerAction| {
      self.player_pending_actions.push(a);
    };
    // parse input, (inputs, ?prev_logic_state?) -> actions
    input_player(&keys_down, &pressed, &mut do_action);
  }
  fn update(&mut self, _: f64, dt_seconds: f64) {
    // dispatch actions through reducer, (prev_logic_state, actions) -> next_logic_state
    player_resolve_actions(&self.player_pending_actions, &mut self.player_physics);
    // step the physics simulation, (prev_phys_state, next_logic_state, time) -> next_phys_state
    self.player_physics.update(dt_seconds);
    // clean up
    self.player_pending_actions.clear();
  }
  fn draw(&mut self, renderer: &mut sdl2::render::Renderer) {
    // TODO camera (flips/scales + translates from world coords to pixel coords)
    draw(renderer, &self.background);
    draw_physics(&self.player_physics, renderer);
  }
}


fn main() {
  // sdl setup
  let sdl_context = sdl2::init().unwrap();
  let _sdl_image_context = sdl2::image::init(sdl2::image::INIT_PNG);
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem.window("SDL2", 640, 480)
    .position_centered()
    .build()
    .unwrap();
  let mut renderer = window.renderer()
    .accelerated().build().unwrap();

  let mut timer = sdl_context.timer().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut prev_keys = HashSet::new();
  let mut last_ticks = timer.ticks();
  let mut ticks_leftover = 0;

  // game init
  let mut world = World::new(&mut renderer);
  let sim_dt = 10;
  let sim_dt_seconds = sim_dt as f64 / 1000.;

  'running: loop {
    let ticks = timer.ticks();
    let time = ticks as f64 / 1000.;
    let dt = ticks - last_ticks;
    last_ticks = ticks;
    ticks_leftover += dt;

    // input
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => break 'running,
        _ => {}
      }
    }
    let keys: HashSet<Keycode> = event_pump.keyboard_state()
      .pressed_scancodes()
      .filter_map(Keycode::from_scancode)
      .collect();
    let pressed = &keys - &prev_keys;
    if keys.contains(&Keycode::Escape) {
      break 'running;
    }
    // let released = &prev_keys - &keys;

    // prepare for drawing
    renderer.set_draw_color(Color::RGBA(0,255,0,255));
    renderer.clear();

    // invoke game logic
    world.input(&keys, &pressed);
    while ticks_leftover >= sim_dt {
      world.update(time, sim_dt_seconds);
      ticks_leftover -= sim_dt
    }
    world.draw(&mut renderer);

    // loop finalizing
    renderer.present();
    prev_keys = keys;
  }
}
