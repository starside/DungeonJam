use std::fs::{File, OpenOptions};
use std::io::{BufReader, ErrorKind, Write};
use serde::{Deserialize, Serialize};
use crate::grid2d;
use crate::grid2d::{Grid2D, RayGridCell};

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