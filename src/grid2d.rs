use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use macroquad::math::{DVec2, IVec2};
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub enum GridCellType {
    #[default]
    Empty,
    Wall
}

#[derive(Serialize, Deserialize)]
pub struct RayGridCell {
    pub cell_type: GridCellType
}

#[derive(Serialize, Deserialize)]
pub struct Grid2D<T> {
    width: usize,
    height: usize,
    cells: Vec<T>
}

impl<T: Serialize + DeserializeOwned> Grid2D<T>{
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

    pub fn cell_at_grid_coords_int(&self, pos: IVec2) -> Option<&T> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i32 || pos.y >= self.height as i32 {
            return None;
        }
        let x= pos.x as usize;
        let y = pos.y as usize;
        self.cells.get(y * self.width + x)
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

    fn save_to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn save_to_file(&self, filename: &str) -> bool {
        let mut file =
            match OpenOptions::new().write(true).open(filename) {
                Ok(f) => {f}
                Err(x) => {
                    match x.kind() {
                        ErrorKind::NotFound => {
                            File::create(filename).unwrap()
                        },
                        _ => {
                            panic!("Failed to open file {}, error {}", filename, x.kind());
                        }
                    }
                }
            };

        let data = self.save_to_string();
        file.write_all(data.as_ref()).expect(format!("Failed to write level to file {}", filename).as_str());
        return true;
    }

    pub fn load_from_file(&mut self, filename: &str){
        let reader =
            match OpenOptions::new().read(true).open(filename) {
                Ok(f) => {
                    BufReader::new(f)
                }
                Err(x) => {
                    panic!("Failed to open file: {} for reading, error {}", filename, x.kind());
                }
            };
        let mut v: Grid2D<T> = serde_json::from_reader(reader).expect("Failed to deserialize level data");
        std::mem::swap(self, &mut v);
    }
}

impl Grid2D<RayGridCell>{
    pub fn randomize(&mut self) {
        self.cells = Vec::new();

        let mut rng = rand::thread_rng();
        let die = Uniform::try_from(1..7).unwrap();
        for _ in 0..self.width * self.height {
            if die.sample(&mut rng) == 3 {
                self.cells.push(RayGridCell { cell_type: GridCellType::Wall });
            }
            else {
                self.cells.push(RayGridCell { cell_type: GridCellType::Empty });
            }
        }
    }
}