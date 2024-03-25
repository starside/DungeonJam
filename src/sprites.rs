use macroquad::color::{Color, YELLOW};
use macroquad::math::{DMat2, DVec2, DVec3};
use macroquad::prelude::mat2;
use crate::fpv::FirstPersonViewer;
use crate::level::Level;

pub enum SpriteType {
    Debug
}

pub struct Sprites {
    pub sp_positions: Vec<DVec2>, // midpoint sprite position
    sp_size: Vec<DVec3>, // Describes width, height of sprite, position is midpoint
    sp_type: Vec<SpriteType>,
    sp_draw_order: Vec<(f64, usize)> // distance, index
}

impl Sprites {
    pub fn new() -> Self {
        Sprites {
            sp_positions: Vec::new(),
            sp_size: Vec::new(),
            sp_type: Vec::new(),
            sp_draw_order: Vec::new()
        }
    }

    pub fn delete_sprite(&mut self, sprite_idx: usize) {
        self.sp_positions.swap_remove(sprite_idx);
        self.sp_size.swap_remove(sprite_idx);
        self.sp_type.swap_remove(sprite_idx);
    }

    pub fn add_sprite(&mut self, pos: DVec2, sprite_type: SpriteType) {
        self.sp_positions.push(pos);
        self.sp_type.push(sprite_type);
        self.sp_size.push(DVec3::from((0.1, 0.1, 0.0))); // x scale, y scale, x offset
        self.sp_draw_order.push((f64::INFINITY, 0));
    }
    pub fn draw_sprites(
        &mut self,
        fpv: &mut FirstPersonViewer,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64)
    {
        let (rw, rh) = (fpv.render_size.0 as usize, fpv.render_size.1 as usize);
        let (w, h) = (fpv.render_size.0 as f64, fpv.render_size.1 as f64);
        let plane = plane_scale*dir.perp();
        let camera_inverse = DMat2::from_cols(plane, dir).inverse();

        // Find visible sprites and sort them by distance
        self.sp_draw_order.clear();
        for (i, sprite) in self.sp_positions.iter().enumerate() {
            let sprite_rel_pos = (*sprite - pos);
            let distance_squared = sprite_rel_pos.dot(sprite_rel_pos);
            let transform = camera_inverse.mul_vec2(sprite_rel_pos);
            if transform.y >= 0.0 { // back plane culling
                self.sp_draw_order.push((distance_squared, i));
            }
        }
        self.sp_draw_order.sort_by(|a,b| {
            b.0.partial_cmp(&a.0).unwrap()
        });

        println!("{:?}", self.sp_draw_order);

        // TODO: Frustum culling
        // TODO: Sort sprites based on distance
        for (sprite, sprite_scale) in self.sp_positions.iter().zip(self.sp_size.iter()) {
            let sprite_rel_pos = *sprite - pos;
            let transform = camera_inverse.mul_vec2(sprite_rel_pos);
            if transform.y >= 0.0 {
                let sprite_screen_y = (h /2.0) * (1.0 + transform.x/ transform.y);

                // Calculate width of the sprite
                let sprite_width = ((sprite_scale.x*w) / transform.y).abs();
                // Calculate the left and right pixel to fill in
                let offset_x = (w/2.0)*(1.0 + sprite_scale.z);
                let draw_start_x = 0.0f64.max((-1.0*sprite_width/2.0) + offset_x) as usize;
                let draw_end_x = (rw - 1).min((sprite_width / 2.0 + offset_x) as usize);

                // Calculate height of sprite
                let sprite_height = sprite_scale.y*(h / transform.y).abs();
                let draw_start_y = 0.0f64.max((-1.0*sprite_height/2.0) + sprite_screen_y) as usize;
                let draw_end_y = (rh - 1).min((sprite_height / 2.0 + sprite_screen_y) as usize);
                //let yc = (fpv.render_size.1 as usize - 1).min(0.0f64.max(sprite_screen_y) as usize);

                let rd = fpv.render_image.get_image_data_mut();

                for y in draw_start_y..=draw_end_y {
                    if transform.y < fpv.z_buffer[y] {
                        for x in draw_start_x..=draw_end_x {
                            let c = (transform.y / fpv.z_buffer[y]) as f32;
                            rd[y*rw + x] = Color::new(c, 1.0, 1.0, 1.0).into();
                        }
                    }
                }
                println!("(draw_start_y, draw_end_y) {:?}, sprite_screen_y {}, sprite_width {}, transform.y {}, draw_start_x {}, draw_end_x {}",
                         (draw_start_y, draw_end_y), sprite_screen_y, sprite_width, transform.y, draw_start_x, draw_end_x);
            }
        }
    }
}