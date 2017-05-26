use common::Vec2;

use render::Renderable;
use physics::MovingObject;

pub struct Entity {
    pub center: Vec2,
    pub rend: Option<Renderable>,
    pub phys: Option<MovingObject>,
}
