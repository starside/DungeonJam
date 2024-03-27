use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Write};
use macroquad::color::{BLACK, colors, GREEN, RED};
use macroquad::input::{get_last_key_pressed, is_mouse_button_down, is_mouse_button_pressed, KeyCode, mouse_position, MouseButton};
use macroquad::math::{DVec3, IVec2, Vec2};
use macroquad::prelude::{clear_background, DVec2};
use macroquad::shapes::draw_circle;
use serde::{Deserialize, Serialize};
use crate::{GameState, grid_viewer, image};
use crate::grid2d::{Grid2D, GridCellType, RayGridCell};
use crate::grid_viewer::draw_grid2d_cell;
use crate::mob::{MobId, Mobs};
use crate::sprites::{Sprites, SpriteType};

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub player_start: (usize, usize),
    pub grid: Grid2D<RayGridCell>,
    pub mob_list: Vec<(i32, i32)>, // For now assume monster type
    filename: String
}

impl Level
{
    pub fn new(level_name: &str, world_width: usize, world_height: usize) -> Self {

        let grid = Grid2D::new(world_width, world_height);
        let mut new_level = Level {
            player_start: (8, 8),
            grid,
            filename: level_name.to_string(),
            mob_list: Vec::new()
        };

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
        let f = &self.filename.clone();
        self.load_from_file(f)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let f = &self.filename;
        self.save_to_file(f)
    }
}

pub struct LevelEditor {
    current_brush_idx: usize
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
                   sprite_manager: &mut Sprites,
                   screen_size: (f32, f32), pos: DVec2, dir: DVec2) -> (Option<(DVec2, DVec2)>, Option<GameState>) {
        let mut new_game_state: Option<GameState> = None;
        let brush_table: [GridCellType; 2] = [
            GridCellType::Empty,
            GridCellType::Wall
        ];
        let current_brush = brush_table[self.current_brush_idx];

        //clear_background(BLACK);
        grid_viewer::draw_grid2d(&world.grid, screen_size);

        let mouse_screen_pos = Vec2::from(mouse_position()).as_dvec2();
        let mouse_world_pos = world.grid.screen_to_grid_coords(mouse_screen_pos, screen_size);

        draw_grid2d_cell(mouse_screen_pos.as_vec2(), current_brush, 1.0, &world.grid, screen_size);

        // Draw start position
        let start_pos_world = world_space_centered_coord((world.player_start.0 as i32,world.player_start.1 as i32), 0.0, 0.0);
        let start_pos_screen = world.grid.grid_to_screen_coords(start_pos_world, screen_size).as_vec2();
        draw_circle(start_pos_screen.x, start_pos_screen.y, 5.0, BLACK);

        // Draw current player position
        let player_screen_coords = world.grid.grid_to_screen_coords(pos, screen_size).as_vec2();
        draw_circle(player_screen_coords.x, player_screen_coords.y, 3.0, colors::GOLD);

        // Draw monster positions
        for mob in mob_manager.mob_list.iter() {
            let s = mob.borrow();
            let p = world.grid.grid_to_screen_coords(s.pos, screen_size).as_vec2();
            let mob_color = match s.is_alive {
                true => {GREEN}
                false => {RED}
            };
            draw_circle(p.x, p.y, 3.0, mob_color);
        }

        if is_mouse_button_pressed(MouseButton::Middle){
            self.current_brush_idx = (self.current_brush_idx + 1) % brush_table.len();
        }

        if is_mouse_button_down(MouseButton::Left){
            world.grid.set_cell_at_grid_coords_int(mouse_world_pos.as_ivec2(), RayGridCell{cell_type: current_brush});
        }

        match get_last_key_pressed() {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::P => {
                        let t = mouse_world_pos.as_uvec2();
                        let t = (t.x as usize, t.y as usize);
                        world.player_start = t;
                    }
                    KeyCode::E => {
                        let new_monster_pos = mouse_world_pos.as_ivec2();
                        if mob_manager.new_monster(new_monster_pos, mob_grid) == true {
                            world.mob_list.push(<(i32, i32)>::from(new_monster_pos));
                        }
                    }
                    KeyCode::K => {
                        let kill_monster_pos = mouse_world_pos.as_ivec2();
                        for mob in mob_manager.mob_list.iter() {
                            let mob = &mut mob.borrow_mut();
                            if mob.pos.as_ivec2() == kill_monster_pos {
                                mob.is_alive = false;
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