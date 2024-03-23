mod raycaster;
mod grid2d;
mod grid_viewer;
mod level;
mod fpv;
mod debug;

use macroquad::miniquad::window;
use macroquad::prelude::*;
use crate::level::{Level, world_space_centered_coord};
use crate::PlayerMode::{Idle, Moving};

enum GameState {
    Debug,
    FirstPerson,
    LevelEditor
}

enum PlayerMode {
    Idle,
    Moving
}

struct PlayerState {
    last_key_pressed: Option<KeyCode>,
    mode: PlayerMode
}
impl PlayerState {
    fn do_idle_state(&self, player_facing: &mut f64, player_pos: &mut(usize, usize)) -> PlayerMode {
        match self.last_key_pressed {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::A | KeyCode::D => { //Turn around
                        if *player_facing > 0.0 {
                            *player_facing = -1.0;
                        } else {
                            *player_facing = 1.0;
                        }
                    }
                    KeyCode::W => { // Move forward
                        let pp = *player_pos;
                        if *player_facing < 0.0 {
                            *player_pos = (pp.0 + 1, pp.1);
                        } else {
                            *player_pos = (pp.0 - 1, pp.1);
                        }

                    }
                    KeyCode::S => { // Move backwards
                        let pp = *player_pos;
                        if *player_facing > 0.0 {
                            *player_pos = (pp.0 + 1, pp.1);
                        } else {
                            *player_pos = (pp.0 - 1, pp.1);
                        }

                    }
                    _ => {}
                }
            }
        }
        Idle
    }

    fn do_moving_state(&self) -> PlayerMode {
        Moving
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut world = Level::new("level.json", 32, 128);

    // Camera plane scaling factor
    let plane_scale = -1.1;

    let mut debug_view = debug::DebugView::default();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(640, 480);

    // Translate player starting position to world vector coords.
    // These are the gameplay variables, the others should not be modified directly
    let mut player_pos = world.player_start;
    let mut player_facing: f64 = 1.0;


    // Level editor
    let mut level_editor = level::LevelEditor::new();

    let mut game_state = GameState::LevelEditor;
    let mut player_state = PlayerState{last_key_pressed: None, mode: Idle};

    loop {
        let screen_size = window::screen_size();

        // Handle player view
        let pos = world_space_centered_coord(player_pos, 0.0, -0.0);
        let dir = player_facing * DVec2::from((-1.0, 0.0));
        let plane = plane_scale*dir.perp();

        match game_state {
            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, screen_size) {
                    //pos = p; // These need fixing
                    //dir = d.normalize();
                    //plane = dir.perp() * plane_scale;
                } else {
                    game_state = GameState::FirstPerson;
                }
            }

            GameState::FirstPerson => {
                // Read last keypress
                let last_key_pressed = get_last_key_pressed();
                player_state.last_key_pressed = last_key_pressed;

                // Draw frame
                clear_background(BLACK);
                first_person_view.draw_view(&world, screen_size, pos, dir, plane);

                // Draw FPS meter
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);

                // Execute state machine
                player_state.mode = match player_state.mode {
                    Idle => {
                        player_state.do_idle_state(&mut player_facing, &mut player_pos)
                    }
                    Moving => {
                        player_state.do_moving_state()
                    }
                };

                match last_key_pressed {
                    None => {}
                    Some(x) => {
                        match &x {
                            KeyCode::F1 => {
                                game_state = GameState::LevelEditor;
                            }
                            KeyCode::F2 => {
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
                    //todo!()
                    //(pos, dir) = x;
                }
                if let Some(x) = new_state {
                    game_state = x;
                }
            }
        }

        next_frame().await
    }
}