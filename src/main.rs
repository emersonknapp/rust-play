#[macro_use]
extern crate serde_derive;
extern crate sdl2;
mod camera;
mod common;
mod render;
mod components;
mod systems;
mod physics;
mod editor;
mod platforms;

use std::time;
use std::path::Path;
use std::sync::mpsc;
use std::sync;
use std::io;
use std::io::Write;
use std::thread;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use common::{Vec2, InputState};
use systems::{create_world, run_systems};
use editor::{Editor, run_editor_systems};
use components::{World};

static REQUEST_WINDOW_WIDTH: u32 = 640;
static REQUEST_WINDOW_HEIGHT: u32 = 480;

#[derive(Debug)]
enum ShellCommand {
  Exit,
  DeleteEntity(usize),
  Save(String),
  Load(String),
}

fn parse_input(input: &str, tx: &mpsc::Sender<ShellCommand>) {
  let mut iter = input.split_whitespace();
  match iter.next() {
    Some(command) => {
      match command {
        "exit" => {
          tx.send(ShellCommand::Exit);
        },
        "delent" => {
          match iter.next() {
            Some(idstr) => {
              match idstr.parse::<usize>() {
                Ok(id) => {
                  tx.send(ShellCommand::DeleteEntity(id));
                },
                Err(_) => {},
              };
            },
            None => {},
          }
        },
        "save" => {
          match iter.next() {
            Some(filename) => {
              tx.send(ShellCommand::Save(filename.to_owned()));
            },
            None => {
              println!("Say a filename");
            }
          }
        },
        "load" => {
          match iter.next() {
            Some(filename) => {
              tx.send(ShellCommand::Load(filename.to_owned()));
            },
            None => {
              println!("Say a filename");
            }
          }
        },
        _ => {
          println!("I didn't understand {}", input);
        },
      };
    },
    None => {},
  };
}

fn shell_system(world: &mut World, cmd: ShellCommand) {
}


fn main() {
  // sdl setup
  let sdl_context = sdl2::init().unwrap();
  let _image_context = sdl2::image::init(sdl2::image::INIT_PNG);
  let ttf_context = sdl2::ttf::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem.window("SDL2", REQUEST_WINDOW_WIDTH, REQUEST_WINDOW_HEIGHT)
    .position_centered()
    .build()
    .unwrap();
  // TODO do i have to do scaling for high dpi?
  let (screen_width, screen_height) = (REQUEST_WINDOW_WIDTH, REQUEST_WINDOW_HEIGHT);
  let mut renderer = window.renderer()
    .accelerated().build().unwrap();
  renderer.set_blend_mode(sdl2::render::BlendMode::Blend);

  let mut font = ttf_context.load_font(Path::new("assets/inconsolata.ttf"), 48).unwrap();

  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut prev_keys = event_pump.keyboard_state()
    .pressed_scancodes()
    .filter_map(Keycode::from_scancode)
    .collect();
  let mut prev_mouse = event_pump.mouse_state();

  // game init
  let mut world = create_world(&mut renderer, Vec2::new(screen_width as f64, screen_height as f64));
  let mut editor = Editor::new();
  // let mut world = World::new(&mut renderer, Vec2::new(640., 480.));

  let sim_dt = time::Duration::from_millis(10);
  let target_frame_time = time::Duration::from_millis(16);
  let mut last_time = time::Instant::now();
  let mut dt_accum = time::Duration::new(0, 0);

  // debug stuff
  let mut frame_counter = 0;
  let mut phys_counter = 0;
  let mut frame_counter_accumulator = time::Duration::new(0, 0);

  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    let mut input = String::new();
    'shell: loop {
      match io::stdin().read_line(&mut input) {
        Ok(n) => {
          parse_input(&input[..], &tx);
        },
        Err(error) => println!("error: {}", error),
      }
      input.clear();
    }
  });

  thread::sleep(target_frame_time);
  print!(">> ");
  io::stdout().flush();
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

    match rx.try_recv() {
      Ok(cmd) => {
        match cmd  {
          ShellCommand::Exit => {
            world.alive = false;
          },
          ShellCommand::DeleteEntity(id) => {
            world.delete_entity(id);
          },
          ShellCommand::Save(filename) => {
            world.save(&filename);
          },
          ShellCommand::Load(filename) => {
            let filename = format!("assets/{}.air", filename);
            match World::from_file(Path::new(&filename), &mut renderer) {
              Ok(w) => {
                world = w;
              },
              _ => {
                println!("No such level found")
              },
            }
          }
        }
        print!(">> ");
        io::stdout().flush();
      },
      Err(_) => {},
    }

    // prepare for drawing
    renderer.set_draw_color(Color::RGBA(0,0,0,255));
    renderer.clear();

    dt_accum = run_systems(&mut world, &input, &mut renderer, sim_dt);
    run_editor_systems(&mut world, &mut editor, &input, &mut renderer, &mut font);

    // loop finalizing
    renderer.present();
    prev_keys = input.keys;
    prev_mouse = input.mouse;

    if !world.alive {
      break 'running;
    }
    // Sleep until next frame
    if let Some(sleep_duration) = target_frame_time.checked_sub(last_time.elapsed()) {
      thread::sleep(sleep_duration);
    }
  }
}
