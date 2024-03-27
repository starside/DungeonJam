mod raycaster;
mod grid2d;
mod grid_viewer;
mod level;
mod fpv;
mod debug;
mod player_movement;
mod physics;
mod sprites;
mod image;
mod mob;

use std::fmt::Pointer;
use std::path::{Path, PathBuf};
use macroquad::miniquad::{start, window};
use macroquad::prelude::*;
use crate::grid2d::GridCellType;
use crate::image::ImageLoader;
use crate::level::{Level, icoords_to_dvec2, ucoords_to_icoords, world_space_centered_coord, ucoords_to_dvec2, apply_boundary_conditions_f64};
use crate::mob::{MagicColor, Mob, Mobs, MobType};
use crate::player_movement::{can_climb_down, can_climb_up, can_stem, can_straddle_drop, has_ceiling, has_floor, is_supported_position, MoveDirection, try_move};
use crate::PlayerMode::{Falling, Idle, Moving};
use crate::sprites::SpriteType;

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

fn calculate_view_dir(rotation_angle: f64, player_facing: f64) -> DVec2{
    let rot2d = DVec2::from((rotation_angle.cos(), player_facing*rotation_angle.sin()));
    let dir = player_facing * DVec2::from((-1.0, 0.0));
    rot2d.rotate(dir)
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
                     mobs: &mut Mobs,
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
                        if self.look_rotation != 0.0 {
                            self.look_rotation = 0.0;
                            Idle
                        } else {
                            if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::WalkForward, facing, level) {
                                self.new_player_pos = Some((new_pos.x, new_pos.y));
                                Moving
                            }else {
                                Idle
                            }
                        }
                    }
                    KeyCode::S => { // Move backwards
                        if self.look_rotation != 0.0 {
                            self.look_rotation = 0.0;
                            Idle
                        } else {
                            if let Some(new_pos) = try_move(player_pos_ivec, MoveDirection::WalkBackward, facing, level) {
                                self.new_player_pos = Some((new_pos.x, new_pos.y));
                                Moving
                            } else {
                                Idle
                            }
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
                    KeyCode::Space => { // attack
                        let shoot_dir = calculate_view_dir(self.look_rotation, *player_facing).normalize();
                        let ppos = ucoords_to_dvec2(player_pos) + DVec2::from((0.5, 0.5)); // center in square
                        let shoot_pos =
                            ppos + 0.25*calculate_view_dir(self.look_rotation, *player_facing).normalize();
                        mobs.new_bullet(shoot_pos, shoot_dir, MagicColor::White);
                        Idle
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

fn move_bullets(m: &mut Mob, last_frame_time: f64, world: &Level) {
    let last_state = m.moving;
    debug_assert!(last_state.is_some());
    match &last_state {
        None => {
            m.is_alive = false;
        }
        Some((start,end,lerp)) => {
            let total_move_distance = end.distance(*start);
            let move_this_frame = m.move_speed * last_frame_time;
            let new_lerp =
                (lerp + (move_this_frame/total_move_distance))
                    .clamp(0.0, 1.0);
            let new_pos = apply_boundary_conditions_f64(
                start.lerp(*end, new_lerp),
                world.grid.get_size());
            let hit_wall = player_movement::is_wall(new_pos.as_ivec2(), &world);
            if new_lerp >= 1.0 || hit_wall {
                m.is_alive = false;
                m.moving = None;
            } else {
                m.moving = Some((*start, *end, new_lerp));
                m.pos = new_pos;
            }
        }
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {

    // Load images
    let mut sprite_images = ImageLoader::new();
    let sprite_image_files = vec![
        "sprites/Bones_shadow1_1.png".to_string(),
        "sprites/Pustules_shadow2_2.png".to_string()
    ];
    sprite_images.load_image_list(&sprite_image_files).await.expect("Failed to load sprite images");
    let mut sprite_manager = sprites::Sprites::new();

    // Create mob manager
    let mut mobs = Mobs::new();

    let max_ray_distance: f64 = 16.0;
    let mut world = Level::new("level.json", 16, 64);
    let (world_width, world_height) = world.grid.get_size();

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
                        player_state.do_idle_state(&mut mobs, &mut player_facing, player_pos, &world)
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

                // Delete mobs marked as dead
                mobs.delete_dead_mobs();

                // Update sprites.  Not really efficient but whatever
                sprite_manager.clear_sprites();
                for m in mobs.mob_list.iter() {
                    match m.mob_type {
                        MobType::Monster(_) => {}
                        MobType::Bullet => {
                            let bullet_scaling = DVec3::from((0.1, 0.1, 0.0));
                            sprite_manager.add_sprite(m.pos, 1 as SpriteType, bullet_scaling)
                        }
                    }
                }

                // Animate mobs
                let last_frame_time = get_frame_time() as f64; // Check if game only calls once per frame
                for m in mobs.mob_list.iter_mut() {
                    match m.mob_type {
                        MobType::Monster(_) => {}
                        MobType::Bullet => {
                            move_bullets(m, last_frame_time, &world);
                        }
                    }
                }

                // Draw frame
                let view_dir = calculate_view_dir(player_state.look_rotation, player_facing);
                first_person_view.draw_view(max_ray_distance, &world, pos, view_dir, plane_scale);
                sprite_manager.draw_sprites(
                    max_ray_distance,
                    &sprite_images,
                    &mut first_person_view,
                    pos,
                    view_dir,
                    player_facing*plane_scale,
                    world_width as f64);
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