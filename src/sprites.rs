use std::cmp::Ordering;
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
        self.sp_size.push(DVec3::from((0.1, 0.1, -0.5))); // x scale, y scale, x offset
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
            match b.0.partial_cmp(&a.0) {
                None => {debug_assert!(false);Ordering::Equal} // Prefer nonsense over crashing
                Some(x) => {x}
            }
        });

        // TODO: Frustum culling
        for (sprite, sprite_scale) in self.sp_draw_order.iter().map(|x| {
                let (_, i) = *x;
                (&self.sp_positions[i], self.sp_size[i])
        }) {
            let sprite_rel_pos = *sprite - pos;
            let transform = camera_inverse.mul_vec2(sprite_rel_pos);
            let sprite_screen_y = (h /2.0) * (1.0 + transform.x/ transform.y);

            // Calculate width of the sprite
            let sprite_width = ((sprite_scale.x*w) / transform.y).abs();
            // Calculate the left and right pixel to fill in
            let offset_x = (w/2.0)*(1.0 + sprite_scale.z);
            let draw_start_x_fp = -1.0*sprite_width/2.0 + offset_x;
            let draw_start_x = 0.0f64.max(draw_start_x_fp) as usize;
            let draw_end_x = (rw - 1).min((sprite_width / 2.0 + offset_x) as usize);
            // Calculate x tex coord start
            let tex_delta_x = 1.0/sprite_width as f32;
            let tex_start_x = if draw_start_x_fp < 0.0 {draw_start_x_fp.abs() as f32 * tex_delta_x} else {0.0f32};

            // Calculate height of sprite
            let sprite_height = sprite_scale.y*(h / transform.y).abs();
            let draw_start_y_fp =(-1.0*sprite_height/2.0) + sprite_screen_y;
            let draw_start_y = 0.0f64.max(draw_start_y_fp) as usize;
            let draw_end_y = (rh - 1).min((sprite_height / 2.0 + sprite_screen_y) as usize);
            // Calculate y tex coord start
            let tex_delta_y = 1.0/sprite_height as f32;
            let tex_start_y = if draw_start_y_fp < 0.0 {draw_start_y_fp.abs() as f32 * tex_delta_y} else {0.0f32};

            let rd = fpv.render_image.get_image_data_mut();

            for y in draw_start_y..=draw_end_y {
                if transform.y < fpv.z_buffer[y] {
                    for x in draw_start_x..=draw_end_x {
                        let xc = tex_start_x + ((x-draw_start_x) as f32) * tex_delta_x;
                        let yc = tex_start_y + ((y-draw_start_y) as f32) * tex_delta_y ;
                        rd[y*rw + x] = Color::new(xc, yc, 1.0, 1.0).into();
                    }
                }
            }
        }
    }
}