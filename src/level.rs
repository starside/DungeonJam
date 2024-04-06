use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Write};

use macroquad::color::{BLACK, colors, GOLD, PINK, RED, SKYBLUE, WHITE};
use macroquad::input::{get_last_key_pressed, is_mouse_button_down, is_mouse_button_pressed, KeyCode, mouse_position, MouseButton};
use macroquad::math::{IVec2, Vec2};
use macroquad::prelude::{DVec2};
use macroquad::shapes::draw_circle;
use macroquad::window::clear_background;
use serde::{Deserialize, Serialize};

use crate::{GameState, grid_viewer};
use crate::grid2d::{Grid2D, WallGridCell};
use crate::grid_viewer::draw_grid2d_cell;
use crate::mob::{MobId, Mobs, MobType};
use crate::mob::MagicColor::{Black, White};

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub player_start: (usize, usize),
    pub win_room: (usize, usize),
    pub grid: Grid2D<WallGridCell>,
    pub mob_list: Vec<(i32, i32)>, // For now assume monster type
    pub flavor_sprites: Option<Vec<(f64, f64, usize)>>,
    filename: Option<String>
}

impl Level
{
    pub fn new(level_name: Option<&str>, world_width: usize, world_height: usize) -> Self {

        let grid = Grid2D::new(world_width, world_height);

        let filename = if let Some(s) = level_name {
            Some(s.to_string())
        } else {
            None
        };

        let mut new_level = Level {
            player_start: (8, 8),
            win_room: (world_width / 4, 0),
            grid,
            filename,
            mob_list: Vec::new(),
            flavor_sprites: None
        };

        if let Some(level_name) = level_name {
            match new_level.load_from_file(level_name) {
                Ok(_) => {
                    println!("Loaded {}", level_name);
                }
                Err(x) => {
                    println!("Level {} could not be loaded ({}), generating random", level_name, x);
                    new_level.grid.zero();
                    match new_level.save_to_file(level_name) {
                        Ok(_) => {
                            println!("Saved random level to {}", level_name);
                        }
                        Err(x) => {
                            println!("Failed to save random data to {} because ({})", level_name, x);
                        }
                    }
                }
            }
        }
        new_level
    }
    pub fn save_to_file(&self, filename: &str) -> Result<(), std::io::Error> {
        let file =
            match OpenOptions::new().write(true).truncate(true).open(filename) {
                Ok(f) => {f}
                Err(x) => {
                    match x.kind() {
                        ErrorKind::NotFound => {
                            File::create(filename)?
                        },
                        _ => {
                            return Err(x);
                        }
                    }
                }
            };

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;
        Ok(())
    }

    pub fn load_from_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let reader =
            match OpenOptions::new().read(true).open(filename) {
                Ok(f) => {
                    BufReader::new(f)
                }
                Err(x) => {
                    return Err(x);
                }
            };
        let mut v: Self = serde_json::from_reader(reader)?;
        std::mem::swap(self, &mut v);
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), std::io::Error> {
        match self.filename.clone() {
            None => {
                return Err(std::io::Error::new(ErrorKind::Other, "No filename provided"));
            }
            Some(f) => {
                self.load_from_file(f.as_str())?;
                Ok(())
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        match self.filename.clone() {
            None => {
                return Err(std::io::Error::new(ErrorKind::Other, "No filename provided"));
            }
            Some(f) => {
                self.save_to_file(f.as_str());
                Ok(())
            }
        }
    }
}

pub struct LevelEditor {
    current_brush_idx: usize
}

pub struct PlayerMap {
    grid: Grid2D<WallGridCell>
}

// Applies boundary conditions.  Clamp vertical, wrap horizontal
pub(crate) fn apply_boundary_conditions_i32(pos: IVec2, world_size: (usize, usize)) -> IVec2 {
    let ws = IVec2::from((world_size.0 as i32, world_size.1 as i32));
    let nx = if pos.x < 0 {
        ws.x - (pos.x.abs() % ws.x)
    } else {
        pos.x % ws.x
    };
    let ny = if pos.y < 0 {
        0
    } else if pos.y >= ws.y {
        ws.y - 1
    } else {
        pos.y
    };
    IVec2::from((nx, ny))
}

pub(crate) fn apply_boundary_conditions_f64(pos: DVec2, world_size: (usize, usize)) -> DVec2 {
    let ws = DVec2::from((world_size.0 as f64, world_size.1 as f64));
    let nx = if pos.x < 0.0 {
        let x_abs = pos.x.abs();
        let nr = (x_abs/ws.x).floor();
        ws.x - (pos.x.abs() - (ws.x*nr))
    } else if pos.x >= ws.x{
        let nr = (pos.x/ws.x).floor();
        pos.x - (ws.x*nr)
    } else {
        pos.x
    };

    let ny = if pos.y < 0.0 {
        0.0
    } else if pos.y >= ws.y {
        ws.y
    } else {
        pos.y
    };

    DVec2::from((nx, ny))
}

pub fn ucoords_to_icoords(x: (usize, usize)) -> (i32, i32) {
    (x.0 as i32, x.1 as i32)
}

pub fn icoords_to_dvec2(pos: (i32, i32)) -> DVec2 {
    DVec2::from((pos.0 as f64, pos.1 as f64))
}

pub fn ucoords_to_dvec2(pos: (usize, usize)) -> DVec2 {
    DVec2::from((pos.0 as f64, pos.1 as f64))
}

pub fn world_space_centered_coord(pos: (i32, i32), x_off: f64, y_off: f64) -> DVec2 {
    icoords_to_dvec2(pos) + 0.5 + DVec2::from((x_off, y_off))
}

impl LevelEditor {
    pub fn new() -> Self {
        LevelEditor {
            current_brush_idx: 1
        }
    }

    pub fn draw_editor(&mut self,
                   world: &mut Level,
                   mob_manager: &mut Mobs,
                   mob_grid: &mut Grid2D<MobId>,
                   screen_size: (f32, f32), pos: DVec2, dir: DVec2) -> (Option<(DVec2, DVec2)>, Option<GameState>) {
        let mut new_game_state: Option<GameState> = None;
        let brush_table: [WallGridCell; 2] = [
            WallGridCell::Empty,
            WallGridCell::Wall
        ];
        let current_brush = brush_table[self.current_brush_idx];

        // Create flavor sprites list
        if world.flavor_sprites.is_none() {
            world.flavor_sprites = Some(Vec::new());
        }

        //clear_background(BLACK);
        grid_viewer::draw_grid2d(&world.grid, screen_size);

        let mouse_screen_pos = Vec2::from(mouse_position()).as_dvec2();
        let mouse_world_pos = world.grid.screen_to_grid_coords(mouse_screen_pos, screen_size);

        draw_grid2d_cell(mouse_screen_pos.as_vec2(), current_brush, 1.0, &world.grid, screen_size);

        // Draw start position
        let start_pos_world = world_space_centered_coord((world.player_start.0 as i32,world.player_start.1 as i32), 0.0, 0.0);
        let start_pos_screen = world.grid.grid_to_screen_coords(start_pos_world, screen_size).as_vec2();
        draw_circle(start_pos_screen.x, start_pos_screen.y, 5.0, BLACK);

        // Draw win position
        let win_pos_world = world_space_centered_coord((world.win_room.0 as i32,world.win_room.1 as i32), 0.0, 0.0);
        let win_pos_screen = world.grid.grid_to_screen_coords(win_pos_world, screen_size).as_vec2();
        draw_circle(win_pos_screen.x, win_pos_screen.y, 5.0, SKYBLUE);

        // Draw current player position
        let player_screen_coords = world.grid.grid_to_screen_coords(pos, screen_size).as_vec2();
        draw_circle(player_screen_coords.x, player_screen_coords.y, 3.0, colors::GOLD);

        // Draw flavor sprites
        if let Some(flavor) = &world.flavor_sprites {
            for &(x, y, sprite_id) in flavor {
                let sc = world.grid.grid_to_screen_coords(DVec2::new(x,y), screen_size).as_vec2();
                draw_circle(sc.x, sc.y, 1.0, colors::RED);
            }
        }

        // Draw monster positions
        for mob in mob_manager.mob_list.iter() {
            let s = mob.borrow();
            let p = world.grid.grid_to_screen_coords(s.get_pos(), screen_size).as_vec2();
            let mob_color = match s.is_alive {
                true => {PINK}
                false => {RED}
            };
            match s.mob_type {
                MobType::Monster(_) => {draw_circle(p.x, p.y, 3.0, mob_color);}
                MobType::Bullet => {draw_circle(p.x, p.y, 1.0, WHITE);}
            }
        }

        if is_mouse_button_pressed(MouseButton::Right){
            self.current_brush_idx = (self.current_brush_idx + 1) % brush_table.len();
        }

        if is_mouse_button_down(MouseButton::Left){
            let cp = mouse_world_pos.as_ivec2();
            world.grid.set_cell_at_grid_coords_int(cp, current_brush);
            // Clear flavor sprites in room
            if current_brush == WallGridCell::Empty {
                if let Some(flavor_sprites) = &mut world.flavor_sprites {
                    flavor_sprites.retain_mut(|(x, y, _)| {
                        let fsc = DVec2::new(*x,*y).as_ivec2();
                        fsc != cp
                    });
                }
            }
        }

        match get_last_key_pressed() {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::Key1 => {
                        let p = mouse_world_pos;
                        if let Some(x) = &mut world.flavor_sprites {
                            x.push((p.x, p.y, 0));
                        }
                    }
                    KeyCode::Key2 => {
                        let p = mouse_world_pos;
                        if let Some(x) = &mut world.flavor_sprites {
                            x.push((p.x, p.y, 1));
                        }
                    }

                    KeyCode::P => {
                        let t = mouse_world_pos.as_uvec2();
                        let t = (t.x as usize, t.y as usize);
                        world.player_start = t;
                    }
                    KeyCode::E => {
                        let new_monster_pos = mouse_world_pos.as_ivec2();
                        if mob_manager.new_monster(new_monster_pos, mob_grid, White) {
                            world.mob_list.push(<(i32, i32)>::from(new_monster_pos));
                        }
                    }
                    KeyCode::R => {
                        let new_monster_pos = mouse_world_pos.as_ivec2();
                        if mob_manager.new_monster(new_monster_pos, mob_grid, Black) {
                            world.mob_list.push(<(i32, i32)>::from(new_monster_pos));
                        }
                    }
                    KeyCode::K => {
                        let kill_monster_pos = mouse_world_pos.as_ivec2();
                        world.mob_list.retain_mut(|mob_pos| {
                            IVec2::from(mob_pos.clone()) != kill_monster_pos
                        });
                        if let Some(mob_to_die) = mob_grid.get_cell_at_grid_coords_int(kill_monster_pos) {
                            match mob_to_die {
                                MobId::Mob(x) => {
                                    let mm = &mut x.borrow_mut();
                                    mm.is_alive = false;
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Escape => {
                        new_game_state = Some(GameState::FirstPerson);
                    }
                    KeyCode::F12 => {
                        if world.save().is_err() {
                            println!("Failed to save level");
                        } else {
                            println!("Saved level");
                        }
                    }
                    KeyCode::F9 => {
                        if world.load().is_err() {
                            println!("Failed to load level");
                        } else {
                            println!("Loaded level");
                        }
                    }
                    _ => {}
                }
            }
        }

        (Some((pos, dir)), new_game_state)
    }
}

impl PlayerMap {
    pub fn new(world_size: (usize, usize)) -> Self {
        let mut x = Grid2D::new(world_size.0, world_size.1);
        x.zero();
        PlayerMap {grid: x}
    }

    pub fn add_marker(&mut self, pos: IVec2) {
        //let pos = apply_boundary_conditions_i32(pos, self.grid.get_size());
        self.grid.set_cell_at_grid_coords_int(pos, WallGridCell::Wall);
    }
    pub fn draw_map(&self,
                       screen_size: (f32, f32),
                       pos: DVec2) -> Option<GameState> {
        let mut new_game_state: Option<GameState> = None;

        let (ww, wh) = self.grid.get_size();
        clear_background(BLACK);
        for y in 0..wh {
            for x in 0..ww {
                let cells = self.grid.get_cells();
                let cell = &cells[y * ww + x];
                let cell_pos = Vec2::from((
                      (x as f32),
                      (y as f32)
                ));
                let pos =  self.grid.grid_to_screen_coords(cell_pos.as_dvec2(), screen_size).as_vec2();
                if *cell == WallGridCell::Wall{
                    //draw_circle(pos.x, pos.y, 2.0, GOLD);
                    draw_grid2d_cell(pos, WallGridCell::Wall, 2.0, &self.grid, screen_size);
                }
            }
        }

        let pos =  world_space_centered_coord(pos.as_ivec2().into(), 0.0, 0.0);
        let pos =  self.grid.grid_to_screen_coords(pos, screen_size).as_vec2();
        draw_circle(pos.x as f32, pos.y as f32, 2.0, RED);

        match get_last_key_pressed() {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::Escape | KeyCode::F1 => {
                        new_game_state = Some(GameState::FirstPerson);
                    }
                    _ => {}
                }
            }
        }

        new_game_state
    }
}