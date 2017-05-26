use common::{Vec2};

pub struct Camera {
    pub fovy: f64,
    pub screen_height: f64,
    pub pos: Vec2,
}
impl Camera {
    pub fn object2screen(&self, object_coord: Vec2, object_pos: Vec2) -> Vec2 {
        self.world2screen(object2world(object_coord, object_pos))
    }
    pub fn world2screen(&self, world_coord: Vec2) -> Vec2 {
        camera2screen(
            world2camera(
                world_coord,
                self.pos
            ),
            self.fovy,
            self.screen_height
        )
    }
}

fn camera2screen(cam_coord: Vec2, fovy: f64, screen_height: f64) -> Vec2 {
    Vec2::new(
        cam_coord.x / fovy * screen_height,
        screen_height - (cam_coord.y / fovy * screen_height)
    )
}

fn world2camera(world_coord: Vec2, cam_pos: Vec2) -> Vec2 {
    Vec2::new(
        world_coord.x - cam_pos.x,
        world_coord.y - cam_pos.y,
    )
}

fn object2world(object_coord: Vec2, object_pos: Vec2) -> Vec2 {
    Vec2::new(
        (object_coord.x + object_pos.x),
        (object_coord.y + object_pos.y)
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let object_coord = Vec2::new(0., 0.);
        let object_pos = Vec2::new(5., 7.);
        let object_scale = Vec2::new(1., 1.);
        let camera_pos = Vec2::new(4., 3.);
        let fovy = 10.;
        let screen_height = 1000.;

        let world_coord = object2world(object_coord, object_pos);
        assert!(world_coord.x == 5.);
        assert!(world_coord.y == 7.);

        let camera_coord = world2camera(world_coord, camera_pos);
        assert!(camera_coord.x == 1.);
        assert!(camera_coord.y == 4.);

        let screen_coord = camera2screen(camera_coord, fovy, screen_height);
        assert!(screen_coord.x == 100.);
        assert!(screen_coord.y == 600.);
    }
}
