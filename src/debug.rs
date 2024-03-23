use macroquad::color::{BLACK, BLUE, RED};
use macroquad::input::{get_last_key_pressed, KeyCode, mouse_position};
use macroquad::math::{DVec2, Vec2};
use macroquad::prelude::{clear_background, draw_circle, draw_line};
use crate::grid_viewer;
use crate::level::Level;
use crate::raycaster::cast_ray;

#[derive(Default)]
pub struct DebugView {
    debug_line: (Vec2, Vec2)
}

impl DebugView {
    // Returns position and ray_direction (not normalized), both in world coordinates
    pub fn draw_debug_view(&mut self, world: &mut Level, screen_size: (f32, f32)) ->
    Option<(DVec2, DVec2)>{
        clear_background(BLACK);
        grid_viewer::draw_grid2d(&world.grid, screen_size);
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
                    KeyCode::Escape => {
                        return None;
                    }
                    KeyCode::P => {
                        let debug_file = "debug_level.json";
                        match world.save_to_file(debug_file) {
                            Ok(_) => {eprintln!("Saved world to {}", debug_file);}
                            Err(x) => {eprintln!("Failed to save world to file {}, {}", debug_file, x);}
                        }
                    }
                    KeyCode::L => {
                        let debug_file = "debug_level.json";
                        match world.load_from_file(debug_file) {
                            Ok(_) => {println!("Loaded world from {}", debug_file);}
                            Err(x) => {println!("Failed to load world from {}, {}", debug_file, x);}
                        }
                    }
                    _ => {}
                }
            }
        }

        draw_line(self.debug_line.0.x, self.debug_line.0.y,self.debug_line.1.x, self.debug_line.1.y, 1.0, BLUE);
        draw_circle(self.debug_line.0.x, self.debug_line.0.y, 7.0, BLUE);

        let ray_dir = world.grid.screen_to_grid_coords((self.debug_line.1 - self.debug_line.0).as_dvec2(), screen_size);
        let ray_start = world.grid.screen_to_grid_coords(self.debug_line.0.as_dvec2(), screen_size);

        let (perp_hit_dist, _, _, _) = cast_ray(&world.grid, &ray_start, &ray_dir);

        let first_step = world.grid.grid_to_screen_coords(ray_start + perp_hit_dist*ray_dir, screen_size).as_vec2();

        draw_circle(first_step.x, first_step.y, 2.0, RED);

        Some((ray_start, ray_dir))
    }
}