#![allow(warnings)]
extern crate rand;

use std::default::Default;
use std::f64::consts::PI;

use fpv::{SpriteId, WallTextureBinding};
use macroquad::color;
use macroquad::math::f64;
use macroquad::miniquad::window;
use macroquad::prelude::*;
use macroquad::telemetry::frame;
use rand::Rng;

use crate::combat::Collision;
use crate::fpv::{FirstPersonViewer, RoomTextureBindings, WallTextureBindings};
use crate::grid2d::{Grid2D, WallGridCell};
use crate::image::ImageLoader;
use crate::level::{
    apply_boundary_conditions_f64, ucoords_to_icoords, world_space_centered_coord, Level,
};
use crate::mob::MagicColor::{Black, White};
use crate::mob::{mob_at_cell, MagicColor, MobData, MobId, MobType, Mobs, MONSTER_HP};
use crate::player_movement::{
    has_floor, is_room_occupiable, is_supported_position, is_wall, try_move, MoveDirection,
    PlayerPosition,
};
use crate::sprites::Sprites;
use crate::GameState::FirstPersonHorizonal;
use crate::PlayerMode::{Falling, Idle, Moving, Turning};

mod combat;
mod debug;
mod fpv;
mod grid2d;
mod grid_viewer;
mod image;
mod level;
mod mob;
mod physics;
mod player_movement;
mod raycaster;
mod sprites;

const RENDER_WIDTH: u16 = 640;
const RENDER_HEIGHT: u16 = 480;

#[derive(PartialEq)]
enum GameState {
    Start,
    Debug,
    FirstPerson,
    FirstPersonHorizonal,
    LevelEditor,
    PlayerMap,
    Win,
    Dead,
}

#[derive(PartialEq)]
enum PlayerMode {
    Idle,
    Moving,
    Falling,
    Turning(f64),
}

struct PlayerState {
    last_key_pressed: Option<KeyCode>,
    mode: PlayerMode,
    look_rotation: f64,
    fire_cooldown: f64,

    new_player_pos: Option<(i32, i32)>,
    lerp: f64,
}

struct HorizontalPlayerState {
    turn_direction: f64,
    look_rotation: f64,
    lerp: f64,
}

fn calculate_view_dir(rotation_angle: f64, player_facing: f64) -> DVec2 {
    let rot2d = DVec2::from((rotation_angle.cos(), player_facing * rotation_angle.sin()));
    let dir = player_facing * DVec2::from((-1.0, 0.0));
    rot2d.rotate(dir)
}

impl HorizontalPlayerState {
    fn player_look_horizonal(&mut self) {
        let look_speed: f64 = 1.5; // Time in seconds to cover range
        let look_range: f64 = 2.0 * 3.14;
        let frame_time = get_frame_time() as f64;

        if is_key_down(KeyCode::A) {
            self.look_rotation += look_range / look_speed * frame_time;
            // Need to use time to animate
        } else if is_key_down(KeyCode::D) {
            self.look_rotation -= look_range / look_speed * frame_time;
        }
    }

    fn player_do_180(&mut self) -> bool {
        let look_speed: f64 = 0.35; // Time in seconds to cover range
        let frame_time = get_frame_time() as f64;

        self.lerp += frame_time / look_speed;
        self.lerp = self.lerp.clamp(0.0, 1.0);
        self.look_rotation = self.turn_direction * PI * self.lerp;
        self.lerp == 1.0
    }
}

impl PlayerState {
    fn player_look(&mut self) {
        let look_up_max: f64 = PI / 2.1;
        let look_down_max: f64 = -PI / 2.1;
        let look_speed: f64 = 1.5; // Time in seconds to cover range
        let look_range: f64 = look_up_max - look_down_max;
        let frame_time = get_frame_time() as f64;

        if is_key_down(KeyCode::Up) {
            self.look_rotation += look_range / look_speed * frame_time; // Need to use time to animate
            self.look_rotation = self.look_rotation.min(look_up_max);
        } else if is_key_down(KeyCode::Down) {
            self.look_rotation -= look_range / look_speed * frame_time;
            self.look_rotation = self.look_rotation.max(look_down_max);
        }
    }
    fn do_idle_state(
        &mut self,
        mobs: &mut Mobs,
        player_facing: &mut f64,
        player_pos: &PlayerPosition,
        level: &Level,
        mob_grid: &Grid2D<MobId>,
        mana_color: &mut MagicColor,
        fire_cooldown: f64,
    ) -> PlayerMode {
        let player_pos_ivec = player_pos.get_pos();

        self.new_player_pos = None;

        let facing = *player_facing as i32;
        let standing_on_mob = !is_room_occupiable(player_pos_ivec + IVec2::new(0, 1), mob_grid);

        if !is_supported_position(player_pos_ivec, level) && !standing_on_mob {
            self.new_player_pos = None;
            return Falling;
        }

        self.player_look();

        let next_state = match self.last_key_pressed {
            None => Idle,
            Some(x) => {
                match &x {
                    KeyCode::A => {
                        //Turn around
                        self.look_rotation = 0.0;
                        Turning(-1.0)
                    }
                    KeyCode::D => {
                        //Turn around
                        self.look_rotation = 0.0;
                        Turning(1.0)
                    }
                    KeyCode::W => {
                        // Move forward
                        if self.look_rotation != 0.0 {
                            self.look_rotation = 0.0;
                            Idle
                        } else {
                            if let Some(new_pos) = try_move(
                                player_pos_ivec,
                                MoveDirection::WalkForward,
                                facing,
                                level,
                                &mob_grid,
                            ) {
                                self.new_player_pos = Some((new_pos.x, new_pos.y));
                                Moving
                            } else {
                                Idle
                            }
                        }
                    }
                    KeyCode::S => {
                        // Move backwards
                        if self.look_rotation != 0.0 {
                            self.look_rotation = 0.0;
                            Idle
                        } else {
                            if let Some(new_pos) = try_move(
                                player_pos_ivec,
                                MoveDirection::WalkBackward,
                                facing,
                                level,
                                &mob_grid,
                            ) {
                                self.new_player_pos = Some((new_pos.x, new_pos.y));
                                Moving
                            } else {
                                Idle
                            }
                        }
                    }
                    KeyCode::Q => {
                        // Move up
                        if let Some(new_pos) = try_move(
                            player_pos_ivec,
                            MoveDirection::ClimbUp,
                            facing,
                            level,
                            &mob_grid,
                        ) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        } else {
                            Idle
                        }
                    }
                    KeyCode::E => {
                        // Move down
                        if let Some(new_pos) = try_move(
                            player_pos_ivec,
                            MoveDirection::ClimbDown,
                            facing,
                            level,
                            &mob_grid,
                        ) {
                            self.new_player_pos = Some((new_pos.x, new_pos.y));
                            Moving
                        } else {
                            Idle
                        }
                    }
                    KeyCode::Space => {
                        // attack
                        if self.fire_cooldown == 0.0 {
                            self.fire_cooldown = fire_cooldown;
                            let shoot_dir =
                                calculate_view_dir(self.look_rotation, *player_facing).normalize();
                            let ppos = player_pos.get_pos_dvec() + DVec2::from((0.5, 0.5)); // center in square
                            mobs.new_bullet(ppos, shoot_dir, *mana_color);
                        }
                        Idle
                    }
                    KeyCode::Left => {
                        *mana_color = match mana_color {
                            White => Black,
                            Black => White,
                        };
                        Idle
                    }
                    _ => Idle,
                }
            }
        };

        if next_state == Moving {
            self.lerp = 0.0;
        }

        next_state
    }

    fn do_moving_state(
        &mut self,
        player_pos: &mut PlayerPosition,
        player_world_coord: &mut DVec2,
        mob_grid: &mut Grid2D<MobId>,
        mana_color: &mut MagicColor,
        level: &Level,
    ) -> PlayerMode {
        match self.new_player_pos {
            None => Idle,
            Some(x) => {
                let p = player_pos.get_pos_ituple();
                let begin_pos = world_space_centered_coord(p, 0.0, 0.0);
                let final_pos = world_space_centered_coord(x, 0.0, 0.0);
                let v = final_pos - begin_pos;
                self.lerp += (get_frame_time() / 0.25) as f64;
                self.lerp = self.lerp.min(1.0);
                let upc = begin_pos + self.lerp * v;
                *player_world_coord = DVec2::from(level::apply_boundary_conditions_f64(
                    upc,
                    level.grid.get_size(),
                ));
                if self.lerp == 1.0 {
                    let nc =
                        level::apply_boundary_conditions_i32(IVec2::from(x), level.grid.get_size());
                    let res = player_pos.set_pos(nc, mob_grid);
                    if res.is_err() {
                        eprintln!("Moved player to occupied mob position");
                    }
                    self.new_player_pos = None;
                    Idle
                } else {
                    if let Some(x) = get_last_key_pressed() {
                        if x == KeyCode::Left {
                            *mana_color = mana_color.get_opposite();
                        }
                    }
                    Moving
                }
            }
        }
    }

    fn do_falling_state(
        &mut self,
        player_pos: &mut PlayerPosition,
        player_world_coord: &mut DVec2,
        mob_grid: &mut Grid2D<MobId>,
        level: &Level,
    ) -> PlayerMode {
        let player_icoords = player_pos.get_pos_ituple();
        let player_pos_ivec = IVec2::from(player_icoords);

        self.player_look();

        match self.new_player_pos {
            None => {
                if has_floor(player_pos_ivec, level).is_some() || // Fall stopped by floor
                    !is_room_occupiable(player_pos_ivec + IVec2::new(0, 1), mob_grid)
                {
                    // Fall stopped by mob
                    Idle
                } else {
                    self.new_player_pos = Some((player_icoords.0, player_icoords.1 + 1)); // Fall down one tile
                    self.lerp = 0.0;
                    Falling
                }
            }
            Some(x) => {
                let p = player_pos.get_pos_ituple();
                let begin_pos = world_space_centered_coord(p, 0.0, 0.0);
                let final_pos = world_space_centered_coord(x, 0.0, 0.0);
                let v = final_pos - begin_pos;
                self.lerp += (get_frame_time() / 0.125) as f64;
                self.lerp = self.lerp.min(1.0);
                let upc = begin_pos + self.lerp * v;
                *player_world_coord = DVec2::from(level::apply_boundary_conditions_f64(
                    upc,
                    level.grid.get_size(),
                ));
                if self.lerp == 1.0 {
                    let nc =
                        level::apply_boundary_conditions_i32(IVec2::from(x), level.grid.get_size());
                    let res = player_pos.set_pos(nc, mob_grid);
                    if res.is_err() {
                        eprintln!("Player fell through occupied mob position");
                    }
                    self.new_player_pos = None;
                }
                Falling
            }
        }
    }
}

fn move_bullets(
    bullet: &mut MobData,
    last_frame_time: f64,
    world: &Level,
    mob_grid: &Grid2D<MobId>,
    collisions: &mut Vec<Collision>,
) {
    let last_state = bullet.moving;
    let ws = world.grid.get_size();
    debug_assert!(last_state.is_some());
    match &last_state {
        None => {
            bullet.is_alive = false;
        }
        Some((start, end, lerp)) => {
            let total_move_distance = end.distance(*start);
            let move_this_frame = bullet.move_speed * last_frame_time;
            let new_lerp = (lerp + (move_this_frame / total_move_distance)).clamp(0.0, 1.0);
            let new_pos =
                apply_boundary_conditions_f64(start.lerp(*end, new_lerp), world.grid.get_size());

            // Check for hit with entity
            let mob_hit_by_bullet = mob_at_cell(new_pos.as_ivec2(), mob_grid);
            match mob_hit_by_bullet {
                MobId::NoMob => {}
                MobId::Mob(_) => {
                    bullet.is_alive = false;
                    collisions.push(Collision::new_with_bullet(
                        mob_hit_by_bullet.clone(),
                        bullet.get_color(),
                    ));
                }
                MobId::Player => {
                    collisions.push(Collision::new_with_bullet(
                        MobId::Player,
                        bullet.get_color(),
                    ));
                    bullet.is_alive = false;
                }
            }

            // Check for wall hit or end of movement
            let hit_wall = player_movement::is_wall(new_pos.as_ivec2(), &world);
            if new_lerp >= 1.0 || hit_wall {
                bullet.is_alive = false;
                bullet.moving = None;
            } else {
                bullet.moving = Some((*start, *end, new_lerp));
                bullet.set_pos(new_pos, ws);
            }
        }
    }
}

fn mana_color_srpite_id(magic_color: MagicColor) -> usize {
    match magic_color {
        White => 1,
        Black => 2,
    }
}

fn render_sprite_full_screen(
    sprite_id: usize,
    sprite_manager: &mut Sprites,
    sprite_images: &ImageLoader,
    first_person_view: &mut FirstPersonViewer,
    screen_size: (f32, f32),
) {
    sprite_manager.clear_sprites();
    sprite_manager.add_sprite(
        DVec2::new(2.0, 1.0),
        (sprite_id, White),
        DVec4::new(1.0, 1.0, 0.0, 0.0),
    );

    sprite_manager.draw_sprites(
        5.0,
        &sprite_images,
        first_person_view,
        DVec2::new(1.0, 1.0),
        DVec2::new(1.0, 0.0),
        1.0,
        1.0,
    ); //world_width as f64);
    first_person_view.reset_z_buffer();
    first_person_view.render(screen_size);
    sprite_manager.clear_sprites();
}

fn draw_first_person_frame(
    player_state: &PlayerState,
    player_facing: f64,
    max_ray_distance: f64,
    world: &Level,
    pos: DVec2,
    plane_scale: f64,
    sprite_images: &ImageLoader,
    hide_floors_and_ceiling: bool,
    hide_walls: bool,
    line_width_scale: f64,
    first_person_view: &mut FirstPersonViewer,
    sprite_manager: &mut Sprites,
) {
    let world_size = world.grid.get_size();

    // Draw frame
    let view_dir = calculate_view_dir(player_state.look_rotation, player_facing);
    let texture_bindings = RoomTextureBindings {
        floor: 7,
        wall: 9,
        ceiling: 8,
    };
    let wall_bindings = WallTextureBindings {
        left: WallTextureBinding {
            sprite_id: 10,
            repeat_speed: 16.0,
            pin: true,
        },
        right: WallTextureBinding {
            sprite_id: 11,
            repeat_speed: 16.0,
            pin: true,
        },
    };
    first_person_view.draw_view(
        max_ray_distance,
        &world,
        pos,
        view_dir,
        plane_scale,
        &texture_bindings,
        &wall_bindings,
        &sprite_images,
        hide_floors_and_ceiling,
        hide_walls,
        line_width_scale,
    );
    sprite_manager.draw_sprites(
        max_ray_distance,
        &sprite_images,
        first_person_view,
        pos,
        view_dir,
        player_facing * plane_scale,
        world_size.0 as f64,
    );
}

#[macroquad::main("BasicShapes")]
async fn main() {
    // Load images
    let mut sprite_images = ImageLoader::new();

    let mut game_image_files = vec![
        "sprites/Bones_shadow1_1.png".to_string(), //0
        "sprites/light.png".to_string(),           //1
        "sprites/dark.png".to_string(),            //2
        "sprites/space_ship.png".to_string(),      //3
        "sprites/startscreen.png".to_string(),     //4
        "sprites/win.png".to_string(),             //5
        "sprites/fail.png".to_string(),            //6
        "sprites/steelfloor7.png".to_string(),     //7
        "sprites/lightwall.png".to_string(),       //8
        "sprites/pezwall.png".to_string(),         //9
        "sprites/sf5.png".to_string(),             //10
        "sprites/sf6.png".to_string(),             //11
    ];

    let mut flavor_image_files = vec![
        "sprites/lamp.png".to_string(),
        "sprites/Tentacle_plant_shadow1_1.png".to_string(),
    ];

    let mut flavor_sprite_scaling: Vec<DVec4> = Vec::new();
    flavor_sprite_scaling.push(DVec4::new(0.2, 0.2, 0.0, 0.0));
    flavor_sprite_scaling.push(DVec4::new(0.6, 0.6, 0.0, 0.0));

    let mut sprite_image_files: Vec<String> = Vec::new();
    let flavor_sprites_start_index = game_image_files.len();
    sprite_image_files.append(&mut game_image_files);
    sprite_image_files.append(&mut flavor_image_files);

    sprite_images
        .load_image_list(&sprite_image_files)
        .await
        .expect("Failed to load sprite images");
    let mut sprite_manager = sprites::Sprites::new();

    // Create mob manager
    let mut mobs = Mobs::new();

    let max_ray_distance: f64 = 16.0;
    let (world_width, world_height) = (16usize, 64usize);
    let mut world = Level::new(Some("level.json"), world_width, world_height);

    // Mob grid
    let mut mob_grid: Grid2D<MobId> = Grid2D::new(world_width, world_height);
    mob_grid.zero();

    // Populate world with mobs
    for m in &world.mob_list {
        mobs.new_monster(IVec2::from(*m), &mut mob_grid, MagicColor::White);
    }

    // Camera plane scaling factor
    let plane_scale: f64 = -1.15;
    let horizontal_line_scale = 0.5;

    let mut debug_view = debug::DebugView::default();

    // Set up low resolution renderer
    let mut first_person_view = fpv::FirstPersonViewer::new(RENDER_WIDTH, RENDER_HEIGHT);
    let mut first_person_view_horizontal = fpv::FirstPersonViewer::new(RENDER_WIDTH, RENDER_HEIGHT);

    // Translate player starting position to world vector coords.
    // These are the gameplay variables, the others should not be modified directly
    let mut player_pos = player_movement::PlayerPosition::new(world.player_start, &mut mob_grid);
    let mut player_facing: f64 = 1.0;
    let mut mana_color: MagicColor = White;
    let player_max_hp: f64 = 639.0;
    let mut player_hp: f64 = player_max_hp;
    let fire_cooldown = 1.0;

    // Level editor
    let mut level_editor = level::LevelEditor::new();

    // Player map
    let mut player_map = level::PlayerMap::new(world.grid.get_size());

    let mut game_state = GameState::Start;
    let mut player_state = PlayerState {
        last_key_pressed: None,
        mode: Idle,
        look_rotation: 0.0,
        new_player_pos: None,
        lerp: 0.0,
        fire_cooldown: 0.0,
    };
    let mut horizontal_player_state = HorizontalPlayerState {
        turn_direction: 0.0,
        look_rotation: 0.0,
        lerp: 0.0,
    };

    // Array to store collisions
    let mut collisions: Vec<Collision> = Vec::with_capacity(16);
    let mut new_bullets: Vec<(DVec2, DVec2, MagicColor)> = Vec::new();

    let mut full_screen_mode = false;
    set_fullscreen(full_screen_mode);

    loop {
        let screen_size = window::screen_size();
        clear_background(BLACK.into());
        let last_frame_time = get_frame_time() as f64; // Check if game only calls once per frame

        // Handle player view
        let mut pos = world_space_centered_coord(player_pos.get_pos_ituple(), 0.0, -0.0);
        let dir = player_facing * DVec2::from((-1.0, 0.0));

        // Check for win condition
        if player_pos.get_pos() == IVec2::from(ucoords_to_icoords(world.win_room)) {
            game_state = GameState::Win;
        }

        // Check for death
        if player_hp <= 0.0 {
            game_state = GameState::Dead;
        }

        if let Some(k) = get_last_key_pressed() {
            if k == KeyCode::F4 {
                full_screen_mode = !full_screen_mode;
                set_fullscreen(full_screen_mode);
            }
        }

        match game_state {
            GameState::Start => {
                clear_background(BLACK);
                render_sprite_full_screen(
                    4,
                    &mut sprite_manager,
                    &sprite_images,
                    &mut first_person_view,
                    screen_size,
                );

                if let Some(x) = get_last_key_pressed() {
                    if x == KeyCode::Enter {
                        game_state = GameState::FirstPerson;
                    }
                }
            }

            GameState::Win => {
                clear_background(BLACK);
                render_sprite_full_screen(
                    5,
                    &mut sprite_manager,
                    &sprite_images,
                    &mut first_person_view,
                    screen_size,
                );
            }

            GameState::Dead => {
                clear_background(BLACK);
                render_sprite_full_screen(
                    6,
                    &mut sprite_manager,
                    &sprite_images,
                    &mut first_person_view,
                    screen_size,
                );
            }

            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, screen_size) {
                    //pos = p; // These need fixing
                    //dir = d.normalize();
                    //plane = dir.perp() * plane_scale;
                } else {
                    game_state = GameState::FirstPerson;
                }
            }

            GameState::FirstPersonHorizonal => {
                let h_world_size = 2 * (max_ray_distance as i32) + 1;
                let mut h_world = Level::new(None, h_world_size as usize, 1);
                let mut h_world_floor: Vec<Option<SpriteId>> =
                    Vec::with_capacity(h_world_size as usize);
                let mut h_world_ceiling: Vec<Option<SpriteId>> =
                    Vec::with_capacity(h_world_size as usize);
                h_world_floor.resize_with(h_world_size as usize, Default::default);
                h_world_ceiling.resize_with(h_world_size as usize, Default::default);
                h_world.grid.zero();
                let h_pos = world_space_centered_coord((max_ray_distance as i32, 0), 0.0, 0.0);

                let wall_bindings = WallTextureBindings {
                    left: WallTextureBinding {
                        sprite_id: 10,
                        repeat_speed: 16.0,
                        pin: true,
                    },
                    right: WallTextureBinding {
                        sprite_id: 11,
                        repeat_speed: 16.0,
                        pin: true,
                    },
                };

                // Calculate view
                let base_angle: f64 = if player_facing < 0.0 { PI } else { 0.0 };
                let done_turning = horizontal_player_state.player_do_180();
                let view_dir =
                    calculate_view_dir(base_angle + horizontal_player_state.look_rotation, 1.0);
                let h_facing = view_dir.x.signum() as i32;

                // Populate floor array
                let current_pos = player_pos.get_pos();
                let current_pos_world =
                    world_space_centered_coord(<(i32, i32)>::from(current_pos), 0.0, 0.0);
                for i in 0..h_world_size / 2 {
                    let x = current_pos.x + h_facing * i;
                    let floor_cell = world
                        .grid
                        .get_cell_at_grid_coords_int(IVec2::from((x, current_pos.y + 1)));
                    let floor_cell: Option<SpriteId> = match floor_cell {
                        Some(x) => match x {
                            WallGridCell::Empty => None,
                            WallGridCell::Wall => Some(7),
                        },
                        None => None,
                    };
                    h_world_floor[(h_pos.x as i32 + (h_facing * i)) as usize] = floor_cell;

                    // Ceiling
                    let ceiling_cell = world
                        .grid
                        .get_cell_at_grid_coords_int(IVec2::from((x, current_pos.y - 1)));
                    let ceiling_cell: Option<SpriteId> = match ceiling_cell {
                        Some(x) => match x {
                            WallGridCell::Empty => None,
                            WallGridCell::Wall => Some(8),
                        },
                        None => None,
                    };
                    h_world_ceiling[(h_pos.x as i32 + (h_facing * i)) as usize] = ceiling_cell;
                }

                // Draw floors, ceiling, and walls
                first_person_view_horizontal.reset_image_buffer([0, 0, 0, 255]);
                first_person_view_horizontal.draw_view_horizontal(
                    max_ray_distance,
                    &h_world,
                    h_pos,
                    view_dir,
                    0.5 / horizontal_line_scale,
                    None,
                    &h_world_floor,
                    &h_world_ceiling,
                    &sprite_images,
                    false,
                    (0.5) / plane_scale.abs(),
                    &wall_bindings,
                    current_pos_world.y,
                );

                // Re-render old view
                first_person_view.reset_image_buffer([0, 0, 0, 0]);
                let vertical_look_dir = if view_dir.x < 0.0 { 1.0 } else { -1.0 };
                draw_first_person_frame(
                    &player_state,
                    vertical_look_dir,
                    max_ray_distance,
                    &world,
                    pos,
                    plane_scale,
                    &sprite_images,
                    true,
                    true,
                    0.5,
                    &mut first_person_view,
                    &mut sprite_manager,
                );

                // draw vertical scene render
                h_world
                    .grid
                    .set_cell_at_grid_coords_int(IVec2::new(17, 0), WallGridCell::Wall);
                h_world
                    .grid
                    .set_cell_at_grid_coords_int(IVec2::new(15, 0), WallGridCell::Wall);
                first_person_view_horizontal.draw_view_horizontal(
                    max_ray_distance,
                    &h_world,
                    h_pos,
                    view_dir,
                    0.5 / horizontal_line_scale, // If this is less than 1, we need to scale the line width of the output render by the same ampunt
                    Some(&first_person_view),
                    &h_world_floor,
                    &h_world_ceiling,
                    &sprite_images,
                    true,
                    0.5,
                    &wall_bindings,
                    current_pos_world.y,
                );

                first_person_view_horizontal.render(screen_size);

                // Draw FPS meter
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 400.0, 30.0, DARKGRAY);

                if done_turning {
                    player_facing = -1.0 * player_facing;
                    player_state.mode = Idle;
                    game_state = GameState::FirstPerson;
                }
            }

            GameState::FirstPerson => {
                // Read last keypress
                let last_key_pressed = get_last_key_pressed();
                player_state.last_key_pressed = last_key_pressed;

                if last_key_pressed == Some(KeyCode::LeftShift) {
                    game_state = GameState::FirstPersonHorizonal;
                }

                // Cooldown player attack
                player_state.fire_cooldown = player_state.fire_cooldown.clamp(0.0, fire_cooldown);

                // Update map
                player_map.add_marker(player_pos.get_pos());

                // Execute state machine
                player_state.mode = match player_state.mode {
                    Idle => player_state.do_idle_state(
                        &mut mobs,
                        &mut player_facing,
                        &player_pos,
                        &world,
                        &mob_grid,
                        &mut mana_color,
                        fire_cooldown,
                    ),
                    Moving => player_state.do_moving_state(
                        &mut player_pos,
                        &mut pos,
                        &mut mob_grid,
                        &mut mana_color,
                        &mut world,
                    ),
                    Falling => player_state.do_falling_state(
                        &mut player_pos,
                        &mut pos,
                        &mut mob_grid,
                        &mut world,
                    ),
                    Turning(direction) => {
                        game_state = GameState::FirstPersonHorizonal;
                        horizontal_player_state.turn_direction = direction;
                        horizontal_player_state.lerp = 0.0;
                        horizontal_player_state.look_rotation = 0.0;
                        Turning(direction)
                    }
                };

                match last_key_pressed {
                    None => {}
                    Some(x) => match &x {
                        KeyCode::F1 => {
                            game_state = GameState::PlayerMap;
                        }
                        KeyCode::F8 => {
                            game_state = GameState::LevelEditor;
                        }
                        _ => {}
                    },
                }

                // Delete mobs marked as dead
                mobs.delete_dead_mobs(&mut mob_grid);

                // Update sprites.  Not really efficient but whatever
                sprite_manager.clear_sprites();
                for m in mobs.mob_list.iter() {
                    let m = m.borrow();
                    match &m.mob_type {
                        MobType::Monster(_) => {
                            let shields = m.hp / MONSTER_HP;
                            let monster_scaling = DVec4::new(0.6, 0.6, 0.0, shields);
                            sprite_manager.add_sprite(
                                m.get_pos(),
                                (0, m.get_color()),
                                monster_scaling,
                            )
                        }
                        MobType::Bullet => {
                            let bullet_scaling = DVec4::new(0.1, 0.1, 0.0, 0.0);
                            let sprite_type = (mana_color_srpite_id(m.get_color()), m.get_color());
                            sprite_manager.add_sprite(m.get_pos(), sprite_type, bullet_scaling)
                        }
                    }
                }

                // Add win room sprite
                sprite_manager.add_sprite(
                    world_space_centered_coord(ucoords_to_icoords(world.win_room), 0.0, 0.1),
                    (3, MagicColor::Black),
                    DVec4::new(0.9, 0.9, 0.0, 0.0),
                );

                // Add flavor sprites
                if let Some(flavor_sprites) = &world.flavor_sprites {
                    for &(x, y, sprite_id) in flavor_sprites {
                        let sid = sprite_id + flavor_sprites_start_index;
                        if sprite_images.check_image_index(sid) {
                            // Don't crash for missing flavor images
                            let flavor_scaling =
                                if let Some(sca) = flavor_sprite_scaling.get(sprite_id) {
                                    *sca
                                } else {
                                    DVec4::new(0.1, 0.1, 0.0, 0.0)
                                };
                            sprite_manager.add_sprite(
                                DVec2::new(x, y),
                                (sid, White),
                                flavor_scaling,
                            )
                        }
                    }
                }

                // Animate mobs
                for m in mobs.mob_list.iter() {
                    let (is_monster, mut can_attack, can_change_color, mut can_move) = {
                        let mob_type = &mut m.borrow_mut();
                        match &mut mob_type.mob_type {
                            MobType::Monster(monster) => {
                                monster.update(last_frame_time);
                                (
                                    true,
                                    monster.can_attack(),
                                    monster.can_change_color(),
                                    monster.can_move(),
                                )
                            }
                            MobType::Bullet => {
                                move_bullets(
                                    mob_type,
                                    last_frame_time,
                                    &world,
                                    &mob_grid,
                                    &mut collisions,
                                );
                                (false, false, false, false)
                            }
                        }
                    };

                    if can_move && can_attack {
                        // decide on one or the other
                        can_move = rand::random();
                        can_attack = !can_move;
                    }

                    if is_monster && can_move {
                        let mob_type = &mut m.borrow_mut();
                        let mob_pos = mob_type.get_pos().as_ivec2();
                        let dv: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, 1), (0, -1)];
                        let mut room_choices: Vec<IVec2> = Vec::new();
                        for v in dv {
                            let v = IVec2::from(v) + mob_pos;
                            if is_room_occupiable(v, &mob_grid) && !is_wall(v, &world) {
                                room_choices.push(v);
                            }
                        }
                        if !room_choices.is_empty() {
                            let random_room: usize =
                                rand::thread_rng().gen_range(0..room_choices.len());
                            let new_room = room_choices[random_room];
                            mob_type.set_pos_centered(new_room.as_dvec2(), world.grid.get_size()); // Set new mob pos
                            let old_mobid = mob_grid
                                .get_cell_at_grid_coords_int(mob_pos)
                                .unwrap()
                                .clone();
                            mob_grid.set_cell_at_grid_coords_int(new_room, old_mobid);
                            mob_grid.set_cell_at_grid_coords_int(mob_pos, MobId::NoMob);
                            let move_speed_modifier =
                                if mob_type.get_pos().distance(player_pos.get_pos_dvec()) <= 3.0 {
                                    0.5f64
                                } else {
                                    1.0f64
                                };
                            match &mut mob_type.mob_type {
                                MobType::Monster(x) => {
                                    x.start_move_cooldown(move_speed_modifier);
                                }
                                MobType::Bullet => {}
                            }
                        }
                    }

                    // Change the enemy color if we can and are the same as the player
                    let mut change_color: Option<MagicColor> = None;
                    if can_change_color {
                        let mob_type = &m.borrow();
                        match &mob_type.mob_type {
                            MobType::Monster(_) => {
                                if mana_color == mob_type.get_color() {
                                    if can_attack {
                                        change_color = Some(mob_type.get_color().get_opposite());
                                    }
                                }
                            }
                            MobType::Bullet => {}
                        }
                    }

                    if let Some(new_color) = change_color {
                        let mob_type = &mut m.borrow_mut();
                        match &mut mob_type.mob_type {
                            MobType::Monster(_) => {
                                mob_type.set_color(new_color);
                            }
                            MobType::Bullet => {}
                        }
                    }

                    if change_color.is_some() {
                        let mob_type = &mut m.borrow_mut();
                        match &mut mob_type.mob_type {
                            MobType::Monster(monster) => {
                                monster.start_color_change_cooldown();
                            }
                            MobType::Bullet => {}
                        }
                    }

                    let mut fire: Option<(DVec2, DVec2, MagicColor)> = None;
                    if can_attack {
                        let mob_type = &m.borrow();
                        match &mob_type.mob_type {
                            MobType::Monster(_) => {
                                // Check if line of sight blocked by wall
                                if let Some((_, dir_wall)) =
                                    mob_type.has_line_of_sight_with_bc(pos, &world.grid)
                                {
                                    // Check if another monster blocks line of sight.
                                    let x = mob_type.has_line_of_sight_with_bc(pos, &mob_grid);
                                    if let Some((y, dir)) = x {
                                        if let Some(hit) = mob_grid.get_cell_at_grid_coords_int(y) {
                                            match hit {
                                                MobId::NoMob => {}
                                                MobId::Mob(_) => {}
                                                MobId::Player => {
                                                    if dir.dot(dir_wall) > 0.0 {
                                                        fire = Some((
                                                            mob_type.get_pos(),
                                                            dir.normalize(),
                                                            mob_type.get_color(),
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            MobType::Bullet => {}
                        }
                    }

                    if fire.is_some() {
                        let mob_type = &mut m.borrow_mut();
                        match &mut mob_type.mob_type {
                            MobType::Monster(monster) => {
                                monster.start_attack_cooldown();
                            }
                            MobType::Bullet => {}
                        }
                    }

                    if let Some(x) = fire {
                        new_bullets.push(x)
                    }
                }

                // Create new bullets
                for (pos, dir, color) in new_bullets.iter() {
                    mobs.new_bullet(*pos, *dir, *color);
                }
                new_bullets.clear();

                // Handle collisions
                for c in collisions.iter() {
                    c.damage_target(&mut player_hp, player_max_hp, mana_color);
                }
                collisions.clear();

                // Draw frame
                draw_first_person_frame(
                    &player_state,
                    player_facing,
                    max_ray_distance,
                    &world,
                    pos,
                    plane_scale,
                    &sprite_images,
                    false,
                    false,
                    horizontal_line_scale,
                    &mut first_person_view,
                    &mut sprite_manager,
                );

                first_person_view.render(screen_size);

                // Draw FPS meter
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 400.0, 30.0, DARKGRAY);

                // Show UI
                let (ui_color, mana_color_string) = match mana_color {
                    White => (color::WHITE, "Light"),
                    Black => (color::DARKPURPLE, "Void"),
                };
                let font_size = 30.0 * (screen_size.0 / 800.0);
                let font_y_spacing = font_size * 0.6;
                let font_y_padding = font_size * 0.05;
                let health_string = format!("HP: {}/{}", player_hp as i32, player_max_hp as i32);
                let mana_string = format!("Mana type: {}", mana_color_string);
                draw_text(
                    health_string.as_str(),
                    font_size * 0.1,
                    font_y_spacing,
                    font_size,
                    ui_color,
                );
                draw_text(
                    mana_string.as_str(),
                    font_size * 0.1,
                    2.0 * (font_y_spacing + font_y_padding),
                    font_size,
                    ui_color,
                );

                // Decrease weapon cooldown
                player_state.fire_cooldown -= last_frame_time;

                // Actions to set when transitioning to other game states
                if game_state == FirstPersonHorizonal {
                    horizontal_player_state.lerp = 0.0;
                    horizontal_player_state.look_rotation = 0.0;
                }
            }

            GameState::LevelEditor => {
                let (new_position, new_state) = level_editor.draw_editor(
                    &mut world,
                    &mut mobs,
                    &mut mob_grid,
                    screen_size,
                    pos,
                    dir,
                );
                if let Some(x) = new_state {
                    game_state = x;
                }
            }

            GameState::PlayerMap => {
                if let Some(x) = player_map.draw_map(screen_size, player_pos.get_pos_dvec()) {
                    game_state = x;
                }
            }
        }

        next_frame().await
    }
}
