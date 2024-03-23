mod raycaster;
mod grid2d;
mod grid_viewer;
mod level;
mod fpv;

use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use crate::raycaster::{cast_ray, HitSide};
use crate::level::Level;

#[derive(Default)]
struct DebugView {
    debug_line: (Vec2, Vec2)
}

impl DebugView {
    // Returns position and ray_direction (not normalized), both in world coordinates
    fn draw_debug_view(&mut self, world: &mut Level, screen_size: (f32, f32)) ->
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
                    KeyCode::Q => {
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

enum GameState {
    Debug,
    FirstPerson,
    LevelEditor
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let (world_width, world_height):(usize, usize) = (16, 16);
    let mut world = Level::new(world_width, world_height);

    let mut debug_view = DebugView::default();
    let mut pos = DVec2::from((0.0, 0.0));
    let mut dir = DVec2::from((-1.0, 0.0));
    let plane_scale = -0.66;
    let mut plane = plane_scale*dir.perp();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(640, 480);

    // Level editor
    let mut level_editor = level::LevelEditor::new();

    let mut game_state = GameState::Debug;

    loop {
        let size_screen = screen_size();

        match game_state {
            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, size_screen) {
                    pos = p;
                    dir = d.normalize();
                    plane = dir.perp() * plane_scale;
                } else {
                    game_state = GameState::FirstPerson;
                }
            }

            GameState::FirstPerson => {
                clear_background(BLACK);
                first_person_view.draw_view(&world, size_screen, pos, dir, plane);
                // Draw FPS
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
                match get_last_key_pressed() {
                    None => {}
                    Some(x) => {
                        match &x {
                            KeyCode::Q => {
                                game_state = GameState::Debug;
                            }
                            _ => {}
                        }
                    }
                }

            }

            GameState::LevelEditor => {
                level_editor.draw_editor(&mut world, size_screen);
            }
        }

        next_frame().await
    }
}