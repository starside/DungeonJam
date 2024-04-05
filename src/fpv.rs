use crate::grid2d::WallGridCell::{Empty, Wall};
use crate::image::ImageLoader;
use macroquad::color::{Color, BLACK, BLUE, DARKGREEN, GREEN, RED, SKYBLUE, WHITE};
use macroquad::math::{DVec2, Vec2};
use macroquad::miniquad::FilterMode;
use macroquad::prelude::{draw_texture_ex, DrawTextureParams, Image, Texture2D};

use crate::level::{apply_boundary_conditions_f64, ucoords_to_dvec2, Level};
use crate::mob::MagicColor::Black;
use crate::physics::wrap_double_norm;
use crate::raycaster::HitSide::{Horizontal, Vertical};
use crate::raycaster::{cast_ray, HitSide};
use crate::WallGridCell;

pub fn fog_factor(distance: f64, max_distance: f64) -> f64 {
    f64::exp(-(2.0 * distance / max_distance).powi(2))
}

pub struct FirstPersonViewer {
    pub render_size: (u16, u16),
    pub render_image: Image,
    render_texture: Texture2D,
    pub z_buffer: Vec<f64>,
}

pub type SpriteId = usize;
pub struct RoomTextureBindings {
    pub floor: SpriteId,
    pub wall: SpriteId,
    pub ceiling: SpriteId,
}

#[derive(Clone, Copy)]
pub struct WallTextureBindings {
    pub left: SpriteId,
    pub right: SpriteId,
    pub pin: bool,
}

impl FirstPersonViewer {
    pub fn new(width: u16, height: u16) -> Self {
        let render_image = Image::gen_image_color(width, height, BLACK);
        let render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);
        let z_size = height.max(width);
        let mut z_buffer: Vec<f64> = Vec::with_capacity(z_size as usize);
        for _ in 0..z_size as usize {
            z_buffer.push(f64::INFINITY);
        }

        FirstPersonViewer {
            render_size: (width, height),
            render_image,
            render_texture,
            z_buffer,
        }
    }

    pub fn reset_z_buffer(&mut self) {
        for i in self.z_buffer.iter_mut() {
            *i = f64::INFINITY;
        }
    }

    pub fn draw_view(
        &mut self,
        max_ray_distance: f64,
        world: &Level,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64,
        texture_bindings: &RoomTextureBindings,
        wall_texture_bindings: &WallTextureBindings,
        sprite_manager: &ImageLoader,
    ) {
        let plane = plane_scale * dir.perp();
        let (render_width, render_height) = self.render_size;
        let rd = self.render_image.get_image_data_mut();
        let dir_x_sign = dir.x / dir.x.abs();
        let up = if dir.x >= 0.0 { -1.0 } else { 1.0 };

        let mut sprite_data: [&Image; 4] = [
            sprite_manager.get_image(7),
            sprite_manager.get_image(8),
            sprite_manager.get_image(9),
            sprite_manager.get_image(4),
        ];

        let wall_texture_bindings = if dir_x_sign > 0.0 {
            *wall_texture_bindings
        } else {
            WallTextureBindings {
                left: wall_texture_bindings.right,
                right: wall_texture_bindings.left,
                pin: wall_texture_bindings.pin,
            }
        };

        let left_wall_image = sprite_manager.get_image(wall_texture_bindings.left);
        let (left_wall_pixels, left_wall_width, left_wall_height) = (
            left_wall_image.get_image_data(),
            left_wall_image.width as usize,
            left_wall_image.height as usize,
        );

        let right_wall_image = sprite_manager.get_image(wall_texture_bindings.right);
        let (right_wall_pixels, right_wall_width, right_wall_height) = (
            right_wall_image.get_image_data(),
            right_wall_image.width as usize,
            right_wall_image.height as usize,
        );

        for y in 0..(render_height as usize) {
            let camera_y = up * (2.0 * (y as f64) / (render_height as f64) - 1.0);
            let ray_dir_x = dir.x + plane.x * camera_y;
            let ray_dir_y = dir.y + plane.y * camera_y;
            let ray_dir = DVec2::from((ray_dir_x, ray_dir_y));

            let (perp_wall_dist, hit_type, hit_side, map_coord) =
                cast_ray(&world.grid, &pos, &ray_dir, max_ray_distance);
            let w = render_width as i32;
            let line_width = (w as f64 / perp_wall_dist) as i32;
            let draw_start = 0.max((-line_width / 2) + (w / 2)) as usize;
            let draw_end = w.min(line_width / 2 + w / 2) as usize;
            let rw = render_width as usize;

            // Calculate wall_x
            let wall_hit_coord = pos + perp_wall_dist * ray_dir;
            let wall_y: f64 = if hit_side == Vertical {
                wall_hit_coord.y - wall_hit_coord.y.floor()
            } else {
                wall_hit_coord.x - wall_hit_coord.x.floor()
            };

            // Store z buffer
            match hit_type {
                Empty => self.z_buffer[y] = f64::INFINITY,
                Wall => self.z_buffer[y] = perp_wall_dist,
            }

            // Fog and misc variables for textures
            let dist_wall = perp_wall_dist;
            let dist_player = 0.0f64;
            let fog = fog_factor(perp_wall_dist, max_ray_distance) as f32;

            // tex size
            let sid: Option<SpriteId> = match hit_type {
                Empty => None,
                Wall => {
                    match hit_side {
                        Horizontal => {
                            if ray_dir.y > 0.0 {
                                Some(texture_bindings.floor)
                            } else {
                                Some(texture_bindings.ceiling)
                            }
                        }
                        Vertical => Some(texture_bindings.wall), //side
                    }
                }
            };

            if let Some(texture_id) = sid {
                let tex_width_u = sprite_data[texture_id].width as usize;
                let tex_height_u = sprite_data[texture_id].height as usize;
                let tex_width = sprite_data[texture_id].width as f64;
                let tex_height = sprite_data[texture_id].height as f64;

                // Calculate texY
                let mut tex_y = (wall_y * tex_height) as usize;
                //if hit_side == Vertical && ray_dir_x * dir.x < 0.0 {tex_y = tex_height as usize - tex_y - 1;} // The ifs may need to change
                //if hit_side == Horizontal && ray_dir_y * dir.x > 0.0 {tex_y = tex_height as usize - tex_y - 1;}
                //println!("{}", dir.x/dir.x.abs());

                // How much to step
                let step = (tex_width / (line_width as f64));

                // starting texture pos
                let mut tex_pos = (draw_start as i32 - w / 2 + line_width / 2) as f64 * step;

                if hit_type == Wall && hit_side == Horizontal {
                    if ray_dir.y > 0.0 {
                        tex_y = tex_height as usize - tex_y - 1;
                    } else {
                        tex_y = tex_height as usize - tex_y - 1;
                    }
                }

                let sprite_pixels = sprite_data[texture_id].get_image_data();

                for x in draw_start..draw_end {
                    let tex_x = (tex_pos as usize).clamp(0, tex_width_u - 1);
                    tex_pos += step;

                    let cvp = sprite_pixels[tex_y * tex_height_u + tex_x];
                    let cv = Color::from_rgba(cvp[0], cvp[1], cvp[2], cvp[3]).to_vec();

                    let pixel = &mut rd[y * rw + x];
                    *pixel = Color::from_vec(fog * cv).into();
                }
            } else {
                for x in draw_start..draw_end {
                    let cv = BLACK.to_vec();
                    let pixel = &mut rd[y * rw + x];
                    *pixel = Color::from_vec(fog * cv).into();
                }
            }

            // Draw walls
            for x in 0..draw_start {
                let current_dist = w as f64 / (-2.0 * x as f64 + w as f64); // This can be a table
                let weight = (current_dist - dist_player) / (dist_wall - dist_player);

                let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;

                let distx = (current_floor_pos - pos).dot(DVec2::new(dir_x_sign, 0.0));
                let disty = (current_floor_pos - pos).dot(DVec2::new(0.0, 1.0));

                let u = wrap_double_norm(distx / max_ray_distance);
                let v = wrap_double_norm(disty / max_ray_distance);

                // Left wall tex coords
                let left_tex_x = ((left_wall_width - 1) as f64 * u) as usize;
                let left_tex_y = ((left_wall_height - 1) as f64 * v) as usize;

                // Right wall tex coords
                let right_tex_x = ((right_wall_width - 1) as f64 * u) as usize;
                let right_tex_y = ((right_wall_height - 1) as f64 * v) as usize;

                let left_wall_color = left_wall_pixels[left_tex_y * left_wall_width + left_tex_x];
                let right_wall_color =
                    right_wall_pixels[right_tex_y * right_wall_width + right_tex_x];

                rd[y * rw + x] = left_wall_color.into();
                rd[y * rw + (render_width as usize - 1 - x)] = right_wall_color.into();
            }

            // Draw walls
            /*for x in draw_end..render_width as usize {
                let current_dist = w as f64 / (2.0 * x as f64 - w as f64); // This can be a table
                let weight = (current_dist - dist_player) / (dist_wall - dist_player);

                let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;

                let uv = apply_boundary_conditions_f64(current_floor_pos, world.grid.get_size());
                let u = 1.0 - uv.x / world_size.x;
                let v = uv.y / world_size.y;

                let tex_x = ((wall_width as f64 * u) as usize) % wall_width;
                let tex_y = ((wall_height as f64 * v) as usize) % wall_height;


                let cvp = wall_pixels[tex_y * wall_width + tex_x];
                rd[y * rw + x] = cvp.into();
            }*/
        }
    }

    pub fn draw_view_horizontal(
        &mut self,
        max_ray_distance: f64,
        world: &Level,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64,
        sprite_manager: &ImageLoader,
    ) {
        let plane = plane_scale * dir.perp();
        let (render_width, render_height) = self.render_size;
        let rd = self.render_image.get_image_data_mut();

        let world_size = ucoords_to_dvec2(world.grid.get_size()).as_vec2();

        let mut sprite_data: [&Image; 3] = [
            sprite_manager.get_image(0),
            sprite_manager.get_image(1),
            sprite_manager.get_image(2),
        ];

        for x_NEW in 0..(render_width as usize) {
            let x_d = x_NEW as f64;
            let camera_x = (2.0 * x_d / (render_width as f64) - 1.0);
            let ray_dir_x = dir.x + plane.x * camera_x;
            let ray_dir_y = dir.y + plane.y * camera_x;
            let ray_dir = DVec2::from((ray_dir_x, ray_dir_y));

            let (perp_wall_dist, hit_type, hit_side, map_coord) =
                cast_ray(&world.grid, &pos, &ray_dir, max_ray_distance);
            let h = render_height as i32;
            let line_height = (h as f64 / perp_wall_dist) as i32;
            let draw_start = 0.max((-line_height / 2) + (h / 2)) as usize;
            let draw_end = h.min(line_height / 2 + h / 2) as usize;
            let rh = render_height as usize;

            // Calculate wall_x
            let wall_hit_coord = pos + perp_wall_dist * ray_dir;
            let wall_x: f64 = if hit_side == HitSide::Vertical {
                wall_hit_coord.y - wall_hit_coord.y.floor()
            } else {
                wall_hit_coord.x - wall_hit_coord.x.floor()
            };

            // tex size
            let sid = 0;

            let tex_width_u = sprite_data[sid].width as usize;
            let tex_height_u = sprite_data[sid].height as usize;
            let tex_width = sprite_data[sid].width as f64;
            let tex_height = sprite_data[sid].height as f64;

            // Calculate texY
            let mut tex_x = (wall_x * tex_height) as usize;
            if hit_side == Vertical && dir.x < 0.0 {
                tex_x = tex_width as usize - tex_x - 1;
            } // The ifs may need to change
            if hit_side == Horizontal && dir.y > 0.0 {
                tex_x = tex_width as usize - tex_x - 1;
            }
            //println!("{}", dir.x/dir.x.abs());

            // How much to step
            let step = (tex_height / (line_height as f64));

            // starting texture pos
            let mut tex_pos = (draw_start as i32 - h / 2 + line_height / 2) as f64 * step;

            // Store z buffer
            match hit_type {
                WallGridCell::Empty => self.z_buffer[x_NEW] = f64::INFINITY,
                WallGridCell::Wall => self.z_buffer[x_NEW] = perp_wall_dist,
            }

            let dist_wall = perp_wall_dist;
            let dist_player = 0.0f64;

            let fog = fog_factor(perp_wall_dist, max_ray_distance) as f32;

            for y in 0..draw_start {
                let current_dist = h as f64 / (2.0 * y as f64 - h as f64); // This can be a table
                let weight = (current_dist - dist_player) / (dist_wall - dist_player);
                let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;
                let v = 1.0 - current_floor_pos.y as f32 / world_size.y;
                let d = 1.0 - (current_floor_pos.distance(pos) as f32 / world_size.x);
                let c = Color::new(0.8 * v * d, 0.8 * v * d, v * d, 1.0);
                rd[y * render_width as usize + x_NEW] = c.into();
                rd[(render_height as usize - 1 - y) * render_width as usize + x_NEW] = c.into();
            }

            let sprite_pixels = sprite_data[sid].get_image_data();

            for y in draw_start..draw_end {
                let tex_y = (tex_pos as usize).clamp(0, tex_height_u - 1);
                tex_pos += step;

                let cv = if hit_type != Empty {
                    let cvp = sprite_pixels[tex_y * tex_height_u + tex_x];
                    Color::from_rgba(cvp[0], cvp[1], cvp[2], cvp[3]).to_vec()
                } else {
                    BLACK.to_vec()
                };

                let pixel = &mut rd[y * render_width as usize + x_NEW];
                *pixel = Color::from_vec(fog * cv).into();
            }
        }
    }

    pub fn render(&self, screen_size: (f32, f32)) {
        // Update texture
        let render_texture_params = DrawTextureParams {
            dest_size: Some(Vec2::from(screen_size)),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };
        self.render_texture.update(&self.render_image);
        draw_texture_ex(&self.render_texture, 0., 0., WHITE, render_texture_params);
    }
}
