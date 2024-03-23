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
    let mut world = Level::new("level.json", 32, 32);

    // Camera plane scaling factor
    let plane_scale = -0.8;

    let mut debug_view = debug::DebugView::default();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(640, 480);

    // Translate player starting position to world vector coords
    let mut player_pos = world.player_start;
    let mut player_facing: f64 = 1.0;


    // Level editor
    let mut level_editor = level::LevelEditor::new();

    let mut game_state = GameState::LevelEditor;

    loop {
        let screen_size = window::screen_size();

        // Handle player view
        let mut pos = world_space_centered_coord(player_pos, 0.0, -0.2);
        let mut dir = player_facing * DVec2::from((-1.0, 0.0));
        let mut plane = plane_scale*dir.perp();

        match game_state {
            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, screen_size) {
                    pos = p; // These need fixing
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
                                game_state = GameState::LevelEditor;
                            }
                            KeyCode::F2 => {
                                game_state = GameState::Debug;
                            }
                            KeyCode::A | KeyCode::D => { //Turn around
                                if player_facing > 0.0 {
                                    player_facing = -1.0;
                                } else {
                                    player_facing = 1.0;
                                }
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