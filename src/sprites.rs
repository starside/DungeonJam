use std::cmp::Ordering;
use macroquad::color::{Color, YELLOW};
use macroquad::math::{DMat2, DVec2, DVec3};
use macroquad::prelude::mat2;
use crate::fpv::FirstPersonViewer;
use crate::{fpv, image};
use crate::image::ImageLoader;
use crate::level::Level;

pub type SpriteType = image::ImageId;

pub struct Sprites {
    pub sp_positions: Vec<DVec2>, // midpoint sprite position
    sp_size: Vec<DVec3>, // Describes width, height of sprite, position is midpoint
    sp_type: Vec<SpriteType>,
    sp_draw_order: Vec<(f64, usize)> // distance, index
}

fn find_distance_across_boundary(obj: DVec2, pos: DVec2, facing: f64, world_width: f64) -> DVec2{
    let mut diff = obj - pos;
    if facing < 0.0 &&  diff.x > 0.0{ // facing left
        diff.x = -pos.x + obj.x - world_width;
    } else if facing > 0.0 && diff.x < 0.0 {
        diff.x = world_width + obj.x - pos.x;
    }
    diff
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
        self.sp_size.push(DVec3::from((1.0, 1.0, 0.0))); // x scale, y scale, x offset
        self.sp_draw_order.push((f64::INFINITY, 0));
    }

    pub fn draw_sprites(
        &mut self,
        cutoff_distance: f64,
        sprite_images: &ImageLoader,
        fpv: &mut FirstPersonViewer,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64,
        world_width: f64)
    {
        let (rw, rh) = (fpv.render_size.0 as usize, fpv.render_size.1 as usize);
        let (w, h) = (fpv.render_size.0 as f64, fpv.render_size.1 as f64);
        let plane = plane_scale*dir.perp();
        let camera_inverse = DMat2::from_cols(plane, dir).inverse();

        // Find visible sprites and sort them by distance
        self.sp_draw_order.clear();
        let cutoff = cutoff_distance * cutoff_distance;
        for (i, sprite) in self.sp_positions.iter().enumerate() {
            let sprite_rel_pos = find_distance_across_boundary(
                *sprite,
                pos,
                dir.x/(dir.x.abs()),
                world_width
            ); //(*sprite - pos);
            let distance_squared = sprite_rel_pos.dot(sprite_rel_pos);
            let transform = camera_inverse.mul_vec2(sprite_rel_pos);
            if transform.y >= 0.0 && distance_squared < cutoff { // back plane culling + max draw distance
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
        for (sprite, sprite_scale, sprite_image) in self.sp_draw_order.iter().map(|x| {
                let (_, i) = *x;
                (&self.sp_positions[i], self.sp_size[i], sprite_images.get_image(0))
        }) {
            let sprite_rel_pos = find_distance_across_boundary(
                *sprite,
                pos,
                dir.x/(dir.x.abs()),
                world_width
            );
            let transform = camera_inverse.mul_vec2(sprite_rel_pos);
            let sprite_screen_y = (h /2.0) * (1.0 + transform.x/ transform.y);

            // Sprite image size
            let sprite_width_pixels = (sprite_image.width-1) as f32;
            let sprite_height_pixels = (sprite_image.height-1) as f32;

            // Calculate width of the sprite
            let sprite_width = ((sprite_scale.x*w) / transform.y).abs();
            // Calculate the left and right pixel to fill in
            let offset_x = (w/2.0)*(1.0 + sprite_scale.z);
            let draw_start_x_fp = -1.0*sprite_width/2.0 + offset_x;
            let draw_start_x = 0.0f64.max(draw_start_x_fp) as usize;
            let draw_end_x = (rw - 1).min((sprite_width / 2.0 + offset_x) as usize);
            // Calculate x tex coord start
            let tex_delta_x = 1.0/sprite_width as f32 * sprite_width_pixels;
            let tex_start_x = if draw_start_x_fp < 0.0 {draw_start_x_fp.abs() as f32 * tex_delta_x} else {0.0f32};

            // Calculate height of sprite
            let sprite_height = sprite_scale.y*(h / transform.y).abs();
            let draw_start_y_fp =(-1.0*sprite_height/2.0) + sprite_screen_y;
            let draw_start_y = 0.0f64.max(draw_start_y_fp) as usize;
            let draw_end_y = (rh - 1).min((sprite_height / 2.0 + sprite_screen_y) as usize);
            // Calculate y tex coord start
            let tex_delta_y = 1.0/sprite_height as f32 * sprite_height_pixels;
            let tex_start_y = if draw_start_y_fp < 0.0 {draw_start_y_fp.abs() as f32 * tex_delta_y} else {0.0f32};

            let rd = fpv.render_image.get_image_data_mut();
            let sprite_rd = sprite_image.get_image_data();

            let sprite_width_u = sprite_image.width as usize;
            let mut tex_y = tex_start_y;
            for y in draw_start_y..=draw_end_y {
                let mut tex_x = tex_start_x;
                let fog_f32 = fpv::fog_factor(transform.y, cutoff_distance) as f32;
                if transform.y < fpv.z_buffer[y] {
                    for x in draw_start_x..=draw_end_x {
                        let sprite_x = sprite_width_pixels.min(tex_x) as usize;
                        let sprite_y = sprite_height_pixels.min(tex_y) as usize;
                        let s = sprite_rd[sprite_y * sprite_width_u + sprite_x];
                        if s[3] > 8 {
                            rd[y * rw + x] =
                                [   (s[0] as f32 * fog_f32) as u8,
                                    (s[1] as f32 * fog_f32) as u8,
                                    (s[2] as f32 * fog_f32) as u8,
                                    (s[3] as f32 * fog_f32) as u8
                                ];
                        }

                        tex_x += tex_delta_x;
                    }
                }
                tex_y += tex_delta_y;
            }
        }
    }
}