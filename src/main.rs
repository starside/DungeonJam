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
    // Returns position and ray_direction (not normalized), both in world coordinates
    fn draw_debug_view(&mut self, world: &grid2d::Grid2D<grid2d::RayGridCell>, screen_size: (f32, f32))  ->
                                                                                                         Option<(DVec2, DVec2)>{
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
                    KeyCode::Q => {
                        return None;
                    }
                    _ => {}
                }
            }
        }

        draw_line(self.debug_line.0.x, self.debug_line.0.y,self.debug_line.1.x, self.debug_line.1.y, 1.0, BLUE);
        draw_circle(self.debug_line.0.x, self.debug_line.0.y, 7.0, BLUE);

        let ray_dir = world.screen_to_grid_coords((self.debug_line.1 - self.debug_line.0).as_dvec2(), screen_size);
        let ray_start = world.screen_to_grid_coords(self.debug_line.0.as_dvec2(), screen_size);

        let (perp_hit_dist, hit_type) = cast_ray(&world, &ray_start, &ray_dir);

        let first_step = world.grid_to_screen_coords(ray_start + perp_hit_dist*ray_dir, screen_size).as_vec2();

        draw_circle(first_step.x, first_step.y, 2.0, RED);

        Some((ray_start, ray_dir))
    }
}

enum GameState {
    DebugView,
    FirstPersonView
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let (world_width, world_height):(usize, usize) = (16, 16);
    let mut world: grid2d::Grid2D<grid2d::RayGridCell> = grid2d::Grid2D::new(world_width, world_height);
    world.randomize();

    let mut debug_view = DebugView::default();
    let mut pos = DVec2::from((0.0, 0.0));
    let mut dir = DVec2::from((1.0, 0.0));

    // Set up low resolution renderer
    let (render_width, render_height) = (640u16, 480u16);
    let mut render_image = Image::gen_image_color(render_width, render_height, BLACK);
    let render_texture = Texture2D::from_image(&render_image);
    render_texture.set_filter(FilterMode::Nearest);

    let mut game_state = GameState::DebugView;

    loop {
        let size_screen = macroquad::miniquad::window::screen_size();

        match game_state {
            GameState::DebugView => {
                if let Some((p, d)) = debug_view.draw_debug_view(&world, size_screen) {
                    pos = p;
                    dir = d;
                } else {
                    game_state = GameState::FirstPersonView;
                }
            }
            GameState::FirstPersonView => {
                clear_background(BLACK);

                for p in render_image.get_image_data_mut() {
                    *p = RED.into();
                }

                // Update texture
                let render_texture_params = DrawTextureParams {
                    dest_size: Some(Vec2::from(size_screen)),
                    source: None,
                    rotation: 0.0,
                    flip_x: false,
                    flip_y: false,
                    pivot: None
                };
                render_texture.update(&render_image);
                draw_texture_ex(&render_texture, 0., 0., WHITE, render_texture_params);

                // Draw FPS
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
                match get_last_key_pressed() {
                    None => {}
                    Some(x) => {
                        match &x {
                            KeyCode::Q => {
                                game_state = GameState::DebugView;
                            }
                            _ => {}
                        }
                    }
                }

            }
        }

        next_frame().await
    }
}