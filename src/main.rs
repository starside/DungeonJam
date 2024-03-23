mod raycaster;
mod grid2d;
mod grid_viewer;
mod level;
mod fpv;
mod debug;

use macroquad::miniquad::window;
use macroquad::prelude::*;
use crate::level::{Level, world_space_centered_coord};

enum GameState {
    Debug,
    FirstPerson,
    LevelEditor
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut world = Level::new("level.json", 16, 16);

    let mut debug_view = debug::DebugView::default();

    let player_start = world_space_centered_coord(world.player_start, 0.0, 0.0);

    let mut pos = player_start;
    let mut dir = DVec2::from((-1.0, 0.0));
    let plane_scale = -0.66;
    let mut plane = plane_scale*dir.perp();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(640, 480);

    // Level editor
    let mut level_editor = level::LevelEditor::new();

    let mut game_state = GameState::LevelEditor;

    loop {
        let screen_size = window::screen_size();

        match game_state {
            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, screen_size) {
                    pos = p;
                    dir = d.normalize();
                    plane = dir.perp() * plane_scale;
                } else {
                    game_state = GameState::FirstPerson;
                }
            }

            GameState::FirstPerson => {
                clear_background(BLACK);
                first_person_view.draw_view(&world, screen_size, pos, dir, plane);
                // Draw FPS
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
                match get_last_key_pressed() {
                    None => {}
                    Some(x) => {
                        match &x {
                            KeyCode::F1 => {
                                game_state = GameState::Debug;
                            }
                            _ => {}
                        }
                    }
                }

            }

            GameState::LevelEditor => {
                let (new_position, new_state) = level_editor.draw_editor(&mut world, screen_size, pos, dir);
                if let Some(x) = new_position {
                    (pos, dir) = x;
                }
                if let Some(x) = new_state {
                    game_state = x;
                }
            }
        }

        next_frame().await
    }
}