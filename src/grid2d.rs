use macroquad::math::{DVec2, IVec2};
use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Default, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum GridCellType {
    #[default]
    Empty,
    Wall
}

#[derive(Serialize, Deserialize, Default)]
pub struct RayGridCell {
    pub cell_type: GridCellType
}

#[derive(Serialize, Deserialize)]
pub struct Grid2D<T> {
    width: usize,
    height: usize,
    cells: Vec<T>
}

impl<T: Serialize + DeserializeOwned + Default> Grid2D<T>{
    pub fn new(width: usize, height: usize) -> Self {
        Grid2D {
            width,
            height,
            cells: Vec::with_capacity(width*height)
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_cells(&self) -> &Vec<T> {
        &self.cells
    }

    pub fn get_cell_screen_size(&self, screen_size: (f32, f32)) -> Vec2{
        let cell_width: f32 = screen_size.0 / (self.width as f32);
        let cell_height: f32 = screen_size.1 / (self.height as f32);
        Vec2::from((cell_width, cell_height))
    }

    pub fn get_cell_at_grid_coords_int(&self, pos: IVec2) -> Option<&T> {
        if pos.y < 0 || pos.y >= self.height as i32 {
            return None;
        }
        let x = if pos.x < 0 {
            ((self.width as i32) + pos.x) as usize
        } else {
            pos.x as usize % self.width
        };
        let y = pos.y as usize;
        self.cells.get(y * self.width + x)
    }

    pub fn set_cell_at_grid_coords_int(&mut self, pos: IVec2, val: T) -> Option<()> {
        let prev = self.get_cell_at_grid_coords_int(pos);
        if prev.is_none() {
            return None;
        }

        let x= pos.x as usize;
        let y = pos.y as usize;
        self.cells[y * self.width + x] = val;
        Some(())
    }

    pub fn screen_to_grid_coords(&self, pos: DVec2, screen_size: (f32, f32)) -> DVec2 {
        let (sw, sh) = screen_size;
        let cell_w =  sw as f64 / self.width as f64;
        let cell_h = sh as f64 / self.height as f64;
        DVec2 {
            x: pos.x / cell_w,
            y: pos.y / cell_h
        }
    }

    pub fn grid_to_screen_coords(&self, pos: DVec2, screen_size: (f32, f32)) -> DVec2 {
        let (sw, sh) = screen_size;
        let cell_w =  sw as f64 / self.width as f64;
        let cell_h = sh as f64 / self.height as f64;
        DVec2 {
            x: pos.x * cell_w,
            y: pos.y * cell_h
        }
    }

    pub fn zero(&mut self) {
        self.cells = Vec::new();
        for _ in 0..self.width * self.height {
            self.cells.push(T::default());
        }
    }
}
