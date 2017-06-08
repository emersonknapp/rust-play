extern crate sdl2;
mod camera;
mod common;
mod render;
mod tilemap;
mod components;
mod systems;

use std::{time, thread};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use common::{Vec2, InputState};
use systems::{create_world, run_systems};


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
  renderer.set_blend_mode(sdl2::render::BlendMode::Blend);

  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut prev_keys = event_pump.keyboard_state()
    .pressed_scancodes()
    .filter_map(Keycode::from_scancode)
    .collect();
  let mut prev_mouse = event_pump.mouse_state();

  // game init
  let mut world = create_world(&mut renderer, Vec2::new(640., 480.));
  // let mut world = World::new(&mut renderer, Vec2::new(640., 480.));

  let sim_dt = time::Duration::from_millis(10);
  let target_frame_time = time::Duration::from_millis(16);
  let mut last_time = time::Instant::now();
  let mut dt_accum = time::Duration::new(0, 0);

  // debug stuff
  let mut frame_counter = 0;
  let mut phys_counter = 0;
  let mut frame_counter_accumulator = time::Duration::new(0, 0);

  thread::sleep(target_frame_time);
  'running: loop {
    let dt = last_time.elapsed();
    last_time = time::Instant::now();
    assert!(dt >= target_frame_time);
    dt_accum += dt;

    // input
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..} => break 'running,
        _ => {}
      }
    }
    let input = InputState {
      keys: event_pump.keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect(),
      last_keys: prev_keys,
      key_mod: sdl_context.keyboard().mod_state(),
      mouse: event_pump.mouse_state(),
      last_mouse: prev_mouse,
    };
    if input.key_down(&Keycode::Escape) {
      break 'running;
    }

    // prepare for drawing
    renderer.set_draw_color(Color::RGBA(0,0,0,255));
    renderer.clear();

    dt_accum = run_systems(&mut world, &input, &mut renderer, sim_dt);

    // loop finalizing
    renderer.present();
    prev_keys = input.keys;
    prev_mouse = input.mouse;

    // Debug output
    frame_counter += 1;
    frame_counter_accumulator += dt;
    if frame_counter_accumulator.as_secs() > 1 {
      println!("cycles {} {}", frame_counter, phys_counter);
      // world.print_stats();
      println!();

      frame_counter_accumulator -= time::Duration::from_secs(1);
      frame_counter = 0;
      phys_counter = 0;
    }

    // Sleep until next frame
    if let Some(sleep_duration) = target_frame_time.checked_sub(last_time.elapsed()) {
      thread::sleep(sleep_duration);
    }
  }
}
