mod raycaster;
mod grid2d;
mod grid_viewer;
mod level;
mod fpv;
mod debug;
mod player_movement;
mod physics;
mod sprites;

use macroquad::miniquad::window;
use macroquad::prelude::*;
use crate::level::{Level, ucoords_to_icoords, world_space_centered_coord};
use crate::player_movement::{can_climb_down, can_climb_up, can_stem, can_straddle_drop, has_ceiling, has_floor, is_supported_position, MoveDirection, try_move};
use crate::PlayerMode::{Falling, Idle, Moving};

enum GameState {
    Debug,
    FirstPerson,
    LevelEditor
}

#[derive(PartialEq)]
enum PlayerMode {
    Idle,
    Moving,
    Falling
}

struct PlayerState {
    last_key_pressed: Option<KeyCode>,
    mode: PlayerMode,
    look_rotation: f64,

    new_player_pos: Option<(i32, i32)>,
    lerp: f64
}

impl PlayerState {
    fn player_look(&mut self) {
        let look_up_max: f64 = 0.9;
        let look_down_max: f64 = -1.14;
        let look_speed: f64 = 0.5; // Time in seconds to cover range
        let look_range: f64 = look_up_max - look_down_max;
        let frame_time = get_frame_time() as f64;

        if is_key_down(KeyCode::Up) {
            self.look_rotation += look_range/look_speed * frame_time; // Need to use time to animate
            self.look_rotation = self.look_rotation.min(look_up_max);
        } else if is_key_down(KeyCode::Down) {
            self.look_rotation -= look_range/look_speed * frame_time;
            self.look_rotation = self.look_rotation.max(look_down_max);
        }
    }
    fn do_idle_state(&mut self,
                     player_facing: &mut f64,
                     player_pos: (usize, usize),
                     level: &Level) -> PlayerMode {
        let player_pos_ivec = IVec2::from(ucoords_to_icoords(player_pos));

        self.new_player_pos = None;

        let facing = *player_facing as i32;

        if !is_supported_position(player_pos_ivec, level) {
            self.new_player_pos = None;
            return Falling;
        }

        self.player_look();

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
                        if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::WalkForward, facing, level) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        }else {
                            Idle
                        }
                    }
                    KeyCode::S => { // Move backwards
                        if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::WalkBackward, facing, level) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        }else {
                            Idle
                        }
                    }
                    KeyCode::Q => { // Move up
                        if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::ClimbUp, facing, level) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        }else {
                            Idle
                        }
                    }
                    KeyCode::E => { // Move down
                        if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::ClimbDown, facing, level) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        }else {
                            Idle
                        }
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
                let p = ucoords_to_icoords(*player_pos);
                let begin_pos = world_space_centered_coord(p, 0.0, 0.0);
                let final_pos = world_space_centered_coord(x, 0.0, 0.0);
                let v = final_pos - begin_pos;
                self.lerp += (get_frame_time()/0.25) as f64;
                self.lerp = self.lerp.min(1.0);
                let upc= begin_pos + self.lerp * v;
                *player_world_coord = DVec2::from(level::apply_boundary_conditions_f64(upc, level.grid.get_size()));
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

    fn do_falling_state(&mut self,
                       player_pos: &mut (usize, usize),
                       player_world_coord: &mut DVec2,
                       level: &Level) -> PlayerMode {
        let player_icoords = ucoords_to_icoords(*player_pos);
        let player_pos_ivec = IVec2::from(player_icoords);

        self.player_look();

        match self.new_player_pos {
            None => {
                if has_floor(player_pos_ivec, level).is_some() { // Fall was stopped by floor
                    Idle
                } else {
                    self.new_player_pos = Some( (player_icoords.0, player_icoords.1 + 1)); // Fall down one tile
                    self.lerp = 0.0;
                    Falling
                }
            }
            Some(x) => {
                let p = ucoords_to_icoords(*player_pos);
                let begin_pos = world_space_centered_coord(p, 0.0, 0.0);
                let final_pos = world_space_centered_coord(x, 0.0, 0.0);
                let v = final_pos - begin_pos;
                self.lerp += (get_frame_time()/0.125) as f64;
                self.lerp = self.lerp.min(1.0);
                let upc= begin_pos + self.lerp * v;
                *player_world_coord = DVec2::from(level::apply_boundary_conditions_f64(upc, level.grid.get_size()));
                if self.lerp == 1.0 {
                    let nc = level::apply_boundary_conditions_i32(IVec2::from(x), level.grid.get_size());
                    *player_pos = (nc.x as usize, nc.y as usize);
                    self.new_player_pos = None;
                }
                Falling
            }
        }


    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let mut world = Level::new("level.json", 16, 64);

    // Camera plane scaling factor
    let plane_scale = -1.05;

    let mut debug_view = debug::DebugView::default();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(640, 480);

    let mut sprite_manager = sprites::Sprites::new();

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
        clear_background(BLACK.into());

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
                    Falling => {
                        player_state.do_falling_state(&mut player_pos, &mut pos, &mut world)
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
                //clear_background(BLACK);
                let rot2d = DVec2::from((player_state.look_rotation.cos(), player_facing*player_state.look_rotation.sin()));
                let view_dir = rot2d.rotate(dir);
                first_person_view.draw_view(&world, pos, view_dir, plane_scale);
                sprite_manager.draw_sprites(&mut first_person_view, pos, view_dir, player_facing*plane_scale);
                first_person_view.render(screen_size);

                // Draw FPS meter
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
            }

            GameState::LevelEditor => {
                let (new_position, new_state) = level_editor.draw_editor(&mut world, &mut sprite_manager, screen_size, pos, dir);
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