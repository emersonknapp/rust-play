extern crate sdl2;
use self::sdl2::mouse::MouseButton;

use components::{World, DrawObstacleTool};
use common::{InputState, AABB, Vec2};
use camera::Camera;

pub fn obstacle_tool_input(input: &InputState, tool: &mut DrawObstacleTool, camera: &Camera, create: &mut Vec<AABB>) {
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
