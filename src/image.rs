use std::future::Future;
use std::path::{Path, PathBuf};
use macroquad::Error;
use macroquad::prelude::{Image, load_image};

pub struct GameImage {
    image: Image,
    filename: PathBuf
}

pub struct ImageLoader  {
    images: Vec<GameImage>
}

pub type ImageId = usize;

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader {
            images: Vec::new()
        }
    }

    // Loads an image, returns its image ID
    pub async fn load_game_image(&mut self, name: &Path) -> Option<ImageId> {
        match  load_image(name.to_str().unwrap()).await {
            Ok(im) => {
                self.images.push(GameImage {
                    image: im,
                    filename: PathBuf::from(name)
                });
                Some(self.images.len() - 1)
            }
            Err(e) => {
                eprintln!("Failed to load image {}: {}", name.to_str().unwrap(), e);
                None
            }
        }
    }

    pub async fn load_image_list(&mut self, files: &Vec<String>) -> Result<(), String> {
        for f in files {
            let file = Path::new(f.as_str());
            if self.load_game_image(file).await.is_none() {
                return Err(format!("Failed to load image {}", f))
            }
        }
        Ok(())
    }

    pub fn get_image(&self, id: ImageId) -> &Image {
        &self.images[id].image
    }
}