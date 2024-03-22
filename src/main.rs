mod raycaster;
mod grid2d;
mod grid_viewer;

use std::ptr::write;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use crate::raycaster::cast_ray;

#[derive(Default)]
struct DebugView {
    debug_line: (Vec2, Vec2)
}

impl DebugView {
    fn draw_debug_view(&mut self, world: &grid2d::Grid2D<grid2d::RayGridCell>, screen_size: (f32, f32)) {
        clear_background(BLACK);
        grid_viewer::draw_grid2d(&world, screen_size);
        match get_last_key_pressed() {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::S => {
                        self.debug_line.0 = mouse_position().into();
                    },
                    KeyCode::E => {
                        self.debug_line.1 = mouse_position().into();
                    },
                    _ => {}
                }
            }
        }

        draw_line(self.debug_line.0.x, self.debug_line.0.y,self.debug_line.1.x, self.debug_line.1.y, 1.0, BLUE);
        draw_circle(self.debug_line.0.x, self.debug_line.0.y, 7.0, BLUE);

        let ray_dir = world.screen_to_grid_coords((self.debug_line.1 - self.debug_line.0).as_dvec2(), screen_size);
        let ray_start = world.screen_to_grid_coords(self.debug_line.0.as_dvec2(), screen_size);

        let first_step_wc = cast_ray(&world, &ray_start, &ray_dir);

        let first_step = world.grid_to_screen_coords(first_step_wc, screen_size).as_vec2();

        draw_circle(first_step.x, first_step.y, 2.0, RED);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let (world_width, world_height):(usize, usize) = (16, 16);
    let mut world: grid2d::Grid2D<grid2d::RayGridCell> = grid2d::Grid2D::new(world_width, world_height);
    world.randomize();

    let mut debug_view = DebugView::default();

    loop {
        let size_screen = macroquad::miniquad::window::screen_size();
        debug_view.draw_debug_view(&world, size_screen);
        next_frame().await
    }
}