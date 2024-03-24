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

#[derive(PartialEq)]
enum PlayerMode {
    Idle,
    Moving
}

struct PlayerState {
    last_key_pressed: Option<KeyCode>,
    mode: PlayerMode,
    look_rotation: f64,

    new_player_pos: Option<(i32, i32)>,
    lerp: f64
}

impl PlayerState {
    fn do_idle_state(&mut self,
                     player_facing: &mut f64,
                     player_pos: (usize, usize),
                     level: &Level) -> PlayerMode {
        let world_size = level.grid.get_size();

        self.new_player_pos = None;

        let is_looking = is_key_down(KeyCode::LeftShift);
        if is_looking {
            if is_key_down(KeyCode::W) {
                self.look_rotation += 0.01; // Need to use time to animate
            } else if is_key_down(KeyCode::S) {
                self.look_rotation -= 0.01;
            }
        } else {
            self.look_rotation = 0.0; // remove this layer of indirection
        }

        let next_state = match self.last_key_pressed {
            None => {Idle}
            Some(x) => {
                match &x {
                    KeyCode::A | KeyCode::D => { //Turn around
                        if *player_facing > 0.0 {
                            *player_facing = -1.0;
                        } else {
                            *player_facing = 1.0;
                        }
                        Idle
                    }
                    KeyCode::W => { // Move forward
                        if !is_looking {
                            self.new_player_pos = Some(
                                (player_pos.0 as i32 + (-1 * *player_facing as i32),
                                 player_pos.1 as i32));
                            Moving
                        } else {
                            Idle
                        }
                    }
                    KeyCode::S => { // Move backwards
                        if !is_looking {
                            self.new_player_pos = Some(
                                (player_pos.0 as i32 + (*player_facing as i32),
                                 player_pos.1 as i32));
                            Moving
                        } else {
                            Idle
                        }
                    }
                    KeyCode::Q => { // Move up
                        self.new_player_pos = Some(
                            (player_pos.0 as i32,
                             -1 + player_pos.1 as i32));
                        Moving
                    }
                    KeyCode::E => { // Move down
                        self.new_player_pos = Some(
                            (player_pos.0 as i32,
                             1 + player_pos.1 as i32));
                        Moving
                    }
                    _ => {Idle}
                }
            }
        };

        if next_state == Moving {
            self.lerp = 0.0;
        }

        next_state
    }

    fn do_moving_state(&mut self,
                       player_pos: &mut (usize, usize),
                       player_world_coord: &mut DVec2,
                       level: &Level) -> PlayerMode {
        match self.new_player_pos {
            None => {Idle}
            Some(x) => {
                let p = (player_pos.0 as i32, player_pos.1 as i32);
                let begin_pos = world_space_centered_coord(p, 0.0, 0.0);
                let final_pos = world_space_centered_coord(x, 0.0, -0.0);
                let v = final_pos - begin_pos;
                self.lerp += (get_frame_time()/0.25) as f64;
                self.lerp = self.lerp.min(1.0);
                let upc= begin_pos + self.lerp * v;
                let npp = DVec2::from(level::apply_boundary_conditions_f64(upc, level.grid.get_size()));
                *player_world_coord = npp;
                if self.lerp == 1.0 {
                    let nc = level::apply_boundary_conditions_i32(IVec2::from(x), level.grid.get_size());
                    *player_pos = (nc.x as usize, nc.y as usize);
                    self.new_player_pos = None;
                    Idle
                } else {
                    Moving
                }
            }
        }
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut world = Level::new("level.json", 16, 16);

    // Camera plane scaling factor
    let plane_scale = -1.05;

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
    let mut player_state = PlayerState{last_key_pressed: None, mode: Idle, look_rotation: 0.0, new_player_pos: None, lerp: 0.0};

    loop {
        let screen_size = window::screen_size();

        // Handle player view
        let mut pos = world_space_centered_coord((player_pos.0 as i32, player_pos.1 as i32), 0.0, -0.0);
        let dir = player_facing * DVec2::from((-1.0, 0.0));

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

                // Execute state machine
                player_state.mode = match player_state.mode {
                    Idle => {
                        player_state.do_idle_state(&mut player_facing, player_pos, &world)
                    }
                    Moving => {
                        player_state.do_moving_state(&mut player_pos, &mut pos, &mut world)
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

                // Draw frame
                clear_background(BLACK);
                let rot2d = DVec2::from((player_state.look_rotation.cos(), player_facing*player_state.look_rotation.sin()));
                first_person_view.draw_view(&world, screen_size, pos, rot2d.rotate(dir), plane_scale);

                // Draw FPS meter
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
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