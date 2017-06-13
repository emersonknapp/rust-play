extern crate sdl2;
use self::sdl2::mouse::MouseButton;
use self::sdl2::keyboard::Keycode;
use self::sdl2::render::Renderer;
use self::sdl2::pixels::Color;
use self::sdl2::ttf::Font;
use sdl2::render::TextureQuery;

use components::{World};
use common::{InputState, AABB, Vec2};
use camera::Camera;
use render::{draw_rect};


pub struct DrawObstacleTool {
  pub pos: Vec2,
  pub start_pos: Option<Vec2>,
}
impl DrawObstacleTool {
  pub fn new() -> DrawObstacleTool {
    DrawObstacleTool {
      pos: Vec2::new(0., 0.),
      start_pos: None,
    }
  }
}

pub struct Editor {
  obstacle_tool: DrawObstacleTool,
}
impl Editor {
  pub fn new() -> Editor {
    Editor {
      obstacle_tool: DrawObstacleTool::new(),
    }
  }
}

fn obstacle_tool_input(input: &InputState, tool: &mut DrawObstacleTool, camera: &Camera, create: &mut Vec<AABB>) {
  tool.pos = camera.screen2world(input.mouse.x(), input.mouse.y());
  if let Some(start_pos) = tool.start_pos {
    if !input.mouse_down(MouseButton::Left) {
      let bbox = AABB {
        center: (tool.pos + start_pos) / 2.,
        half_size: (tool.pos - start_pos).abs() / 2.,
      };
      println!("create({}, {}) ({}, {})", bbox.center.x, bbox.center.y, bbox.half_size.x, bbox.half_size.y);
      create.push(bbox);
      tool.start_pos = None;
    }
  } else {
    if input.mouse_pressed(MouseButton::Left) {
      tool.start_pos = Some(tool.pos);
    }
  }
}

fn render_obstacle_tool(tool: &DrawObstacleTool, camera: &Camera, renderer: &mut Renderer) {
  if let Some(start_pos) = tool.start_pos {
    draw_rect(renderer, camera,
      Vec2::new(start_pos.x.min(tool.pos.x), start_pos.y.min(tool.pos.y)),
      (start_pos - tool.pos).abs(),
      Color::RGBA(90, 150, 20, 120),
    );
  }

  let dot_size = Vec2::new(0.5, 0.5);
  draw_rect(renderer, camera, tool.pos - dot_size / 2., dot_size, Color::RGBA(255, 0, 255, 255));
}

fn render_editor(world: &Editor, renderer: &mut Renderer, camera: &Camera) {
  render_obstacle_tool(&world.obstacle_tool, camera, renderer);
}

pub fn run_editor_systems(world: &mut World, editor: &mut Editor, input: &InputState, renderer: &mut Renderer, font: &mut Font) {
  let mut create_statics = Vec::new();
  if let Some(camera) = world.cameras.get(&world.current_camera) {
    // Systems that need the camera (screen-space tools)
    obstacle_tool_input(input, &mut editor.obstacle_tool, camera, &mut create_statics);
    render_editor(editor, renderer, camera);

    // Draw ID on each entity
    for id in &world.entities {
      if let Some((p, c)) = world.get_collider_entity(*id) {
        let surface = font.render(&format!("{}", id))
          .blended(Color::RGBA(0, 0, 0, 255)).unwrap();
        let mut texture = renderer.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        let target = camera.to_draw_rect(p - c.half_size, Vec2::new(width as f64 / height as f64 * 1.25, 1.25));
        renderer.copy(&mut texture, None, Some(target)).unwrap();
      }
    }

    // TODO factor out
    if input.key_pressed(&Keycode::P) {
      world.save();
    }
  }
  for bbox in &create_statics {
    world.new_static_obstacle(bbox.center, bbox.half_size * 2.);
  }
}
