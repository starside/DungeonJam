use macroquad::color::{BLACK, BLUE, Color, DARKGREEN, SKYBLUE, WHITE};
use macroquad::math::{DVec2, Vec2};
use macroquad::miniquad::FilterMode;
use macroquad::prelude::{draw_texture_ex, DrawTextureParams, Image, Texture2D};

use crate::level::{Level, ucoords_to_dvec2};
use crate::raycaster::{cast_ray, HitSide};
use crate::WallGridCell;

pub fn fog_factor(distance: f64, max_distance: f64) -> f64 {
    f64::exp(-(2.0*distance/max_distance).powi(2))
}

pub struct FirstPersonViewer {
    pub render_size: (u16, u16),
    pub render_image: Image,
    render_texture: Texture2D,
    pub z_buffer: Vec<f64>
}

impl FirstPersonViewer {
    pub fn new(width: u16, height: u16) -> Self {
        let render_image = Image::gen_image_color(width, height, BLACK);
        let render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);
        let mut z_buffer: Vec<f64> = Vec::with_capacity(height as usize);
        for _ in 0..height as usize {
            z_buffer.push(f64::INFINITY);
        }

        FirstPersonViewer {
            render_size: (width, height),
            render_image,
            render_texture,
            z_buffer
        }
    }
    pub fn draw_view(
        &mut self,
        max_ray_distance: f64,
        world: &Level,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64) {

        let plane = plane_scale*dir.perp();
        let (render_width, render_height) = self.render_size;
        let rd = self.render_image.get_image_data_mut();
        let up = if dir.x >= 0.0 {
            -1.0
        } else {
            1.0
        };

        let world_size = ucoords_to_dvec2(world.grid.get_size()).as_vec2();

        for y in 0..(render_height as usize) {
            let y_d = y as f64;
            let camera_y =  up*(2.0 * y_d / (render_height as f64) - 1.0);
            let ray_dir_x = dir.x + plane.x * camera_y;
            let ray_dir_y = dir.y + plane.y * camera_y;
            let ray_dir = DVec2::from((ray_dir_x, ray_dir_y));

            let (perp_wall_dist, hit_type, hit_side, map_coord)
                = cast_ray(&world.grid, &pos, &ray_dir, max_ray_distance);
            let w = render_width as i32;
            let line_width = (w as f64 / perp_wall_dist) as i32;
            let draw_start = 0.max((-line_width/2) + (w/2)) as usize;
            let draw_end = w.min(line_width / 2 + w / 2) as usize;
            let rw = render_width as usize;

            // Calculate wall_x
            let wall_hit_coord = pos + perp_wall_dist * ray_dir;
            let wall_x: f64 = if hit_side == HitSide::Vertical {
                wall_hit_coord.y - wall_hit_coord.y.floor()
            } else {
                wall_hit_coord.x - wall_hit_coord.x.floor()
            };

            // Store z buffer
            match hit_type {
                WallGridCell::Empty => {self.z_buffer[y] = f64::INFINITY}
                WallGridCell::Wall => {self.z_buffer[y] = perp_wall_dist}
            }

            let dist_wall = perp_wall_dist;
            let dist_player = 0.0f64;

            let fog = fog_factor(perp_wall_dist, max_ray_distance) as f32;
            let color =
                match hit_type {
                    WallGridCell::Empty => { BLACK }
                    WallGridCell::Wall => {
                        match hit_side {
                            HitSide::Horizontal => {
                                if ray_dir.y > 0.0 {
                                    SKYBLUE //top
                                } else {
                                    DARKGREEN // bottom
                                }
                            }
                            HitSide::Vertical => { BLUE } //side
                        }
                    }
                };

            let ww2 = world_size.x.powi(2);

            for x in 0..draw_start {
                let current_dist = w as f64 / (2.0 * x as f64 - w as f64); // This can be a table
                let weight = (current_dist - dist_player) / (dist_wall - dist_player);
                let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;
                let v = 1.0-current_floor_pos.y as f32 / world_size.y;
                let d = 1.0-(current_floor_pos.distance_squared(pos) as f32 / ww2);
                let c = Color::new(0.8*v*d, 0.8*v*d, v*d, 1.0);
                rd[y * rw + x] = c.into();
                rd[y * rw + (render_width-1) as usize - x ] = c.into();
            }

            for x in draw_start..draw_end {
                let cv = Color::to_vec(&color);
                let pixel = &mut rd[y * rw + x];
                *pixel = Color::from_vec(fog * cv).into();
            }
        }


    }

    pub fn render(&self, screen_size: (f32, f32),) {
        // Update texture
        let render_texture_params = DrawTextureParams {
            dest_size: Some(Vec2::from(screen_size)),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None
        };
        self.render_texture.update(&self.render_image);
        draw_texture_ex(&self.render_texture, 0., 0., WHITE, render_texture_params);
    }
}