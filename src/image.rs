use std::future::Future;
use macroquad::Error;
use macroquad::prelude::{Image, load_image};

struct GameImage {
    image: Image,
    filename: String
}

struct ImageLoader  {
    images: Vec<GameImage>
}

type ImageId = usize;

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader {
            images: Vec::new()
        }
    }

    // Loads an image, returns its image ID
    pub async fn load_game_image(&mut self, name: &str) -> Option<ImageId> {
        match  load_image(name).await {
            Ok(im) => {
                self.images.push(GameImage {
                    image: im,
                    filename: name.to_string()
                });
                Some(self.images.len() - 1)
            }
            Err(e) => {
                eprintln!("Failed to load image {}: {}", name, e);
                None
            }
        }
    }

    pub async fn load_image_list(&mut self, files: &Vec<String>) -> Result<(), String> {
        for f in files {
            if self.load_game_image(f).await.is_none() {
                return Err(format!("Failed to load image {}", f))
            }
        }
        Ok(())
    }
}