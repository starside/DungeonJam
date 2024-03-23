use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, LineWriter, Write};
use macroquad::color::BLACK;
use macroquad::input::{get_last_key_pressed, is_mouse_button_down, is_mouse_button_pressed, KeyCode, mouse_position, MouseButton};
use macroquad::math::Vec2;
use macroquad::prelude::{clear_background, DVec2};
use serde::{Deserialize, Serialize};
use crate::{GameState, grid_viewer};
use crate::grid2d::{Grid2D, GridCellType, RayGridCell};
use crate::grid_viewer::draw_grid2d_cell;

#[derive(Serialize, Deserialize)]
pub struct Level {
    player_start: (usize, usize),
    pub grid: Grid2D<RayGridCell>,
    filename: String
}

impl Level
{
    pub fn new(level_name: &str, world_width: usize, world_height: usize) -> Self {

        let mut grid = Grid2D::new(world_width, world_height);
        let mut new_level = Level {
            player_start: (8, 8),
            grid,
            filename: level_name.to_string()
        };

        match new_level.load_from_file(level_name) {
            Ok(_) => {
                println!("Loaded {}", level_name);
            }
            Err(x) => {
                println!("Level {} could not be loaded ({}), generating random", level_name, x);
                new_level.grid.randomize();
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
        let mut file =
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

impl LevelEditor {
    pub fn new() -> Self {
        LevelEditor {
            current_brush_idx: 1
        }
    }

    pub fn draw_editor(&mut self,
                   world: &mut Level,
                   screen_size: (f32, f32), pos: DVec2, dir: DVec2) -> (Option<(DVec2, DVec2)>, Option<GameState>) {
        let mut new_game_state: Option<GameState> = None;
        let brush_table: [GridCellType; 2] = [
            GridCellType::Empty,
            GridCellType::Wall
        ];
        let current_brush = brush_table[self.current_brush_idx];

        clear_background(BLACK);
        grid_viewer::draw_grid2d(&world.grid, screen_size);

        let mouse_screen_pos = Vec2::from(mouse_position()).as_dvec2();
        let mouse_world_pos = world.grid.screen_to_grid_coords(mouse_screen_pos, screen_size);

        draw_grid2d_cell(mouse_screen_pos.as_vec2(), current_brush, 1.0, &world.grid, screen_size);

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
                    KeyCode::Escape => {
                        new_game_state = Some(GameState::Debug);
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