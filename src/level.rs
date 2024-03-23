use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use macroquad::color::BLACK;
use macroquad::input::{is_mouse_button_down, mouse_position, MouseButton};
use macroquad::math::Vec2;
use macroquad::prelude::clear_background;
use serde::{Deserialize, Serialize};
use crate::{grid2d, grid_viewer};
use crate::grid2d::{Grid2D, GridCellType, RayGridCell};

#[derive(Serialize, Deserialize)]
pub struct Level {
    player_start: (usize, usize),
    pub grid: Grid2D<RayGridCell>
}

impl Level
{
    pub fn new(world_width: usize, world_height: usize) -> Self {
        let mut grid = Grid2D::new(world_width, world_height);
        grid.randomize();

        Level {
            player_start: (8, 8),
            grid
        }
    }
    pub fn save_to_file(&self, filename: &str) -> Result<(), std::io::Error> {
        let mut file =
            match OpenOptions::new().write(true).open(filename) {
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

        let data = serde_json::to_string(self)?;
        file.write_all(data.as_ref())?;
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
}

pub struct LevelEditor {
    current_brush: GridCellType
}

impl LevelEditor {
    pub fn new() -> Self {
        LevelEditor {
            current_brush: GridCellType::Wall
        }
    }

    pub fn draw_editor(&mut self,
                   world: &mut Level,
                   screen_size: (f32, f32)) { //, pos: DVec2, dir: DVec2) {
        clear_background(BLACK);
        grid_viewer::draw_grid2d(&world.grid, screen_size);

        let mouse_screen_pos = Vec2::from(mouse_position()).as_dvec2();
        let mouse_world_pos = world.grid.screen_to_grid_coords(mouse_screen_pos, screen_size);
        if is_mouse_button_down(MouseButton::Left){
            println!("PRessed {}", mouse_world_pos);
        }
    }
}