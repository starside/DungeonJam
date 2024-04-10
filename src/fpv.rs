use crate::grid2d::WallGridCell::{Empty, Wall};
use crate::image::ImageLoader;
use macroquad::color::{Color, BLACK, BLUE, DARKGREEN, GREEN, RED, SKYBLUE, WHITE};
use macroquad::input::KeyCode::P;
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
pub struct WallTextureBinding {
    pub sprite_id: SpriteId,
    pub repeat_speed: f64,
    pub pin: bool,
}

#[derive(Clone, Copy)]
pub struct WallTextureBindings {
    pub left: WallTextureBinding,
    pub right: WallTextureBinding,
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

    pub fn reset_image_buffer(&mut self, color: [u8;4]) {
        for i in self.render_image.get_image_data_mut() {
            *i = color;
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
        hide_floor_ceiling: bool,
        hide_walls: bool,
        line_width_scale: f64
    ) {
        let world_size = ucoords_to_dvec2(world.grid.get_size());
        let plane = plane_scale * dir.perp();
        let (render_width, render_height) = self.render_size;
        let rd = self.render_image.get_image_data_mut();
        let dir_x_sign = dir.x / dir.x.abs();
        let up = if dir.x >= 0.0 { -1.0 } else { 1.0 };

        // Independent walls are not yet supported
        debug_assert_eq!(
            wall_texture_bindings.left.repeat_speed,
            wall_texture_bindings.right.repeat_speed
        );
        debug_assert_eq!(
            wall_texture_bindings.left.pin,
            wall_texture_bindings.right.pin
        );

        let wall_pin = wall_texture_bindings.left.pin;
        let wall_speed = wall_texture_bindings.left.repeat_speed;

        let wall_texture_bindings = if dir_x_sign > 0.0 {
            *wall_texture_bindings
        } else {
            WallTextureBindings {
                left: wall_texture_bindings.right,
                right: wall_texture_bindings.left,
            }
        };

        let left_wall_image = sprite_manager.get_image(wall_texture_bindings.left.sprite_id);
        let (left_wall_pixels, left_wall_width, left_wall_height) = (
            left_wall_image.get_image_data(),
            left_wall_image.width as usize,
            left_wall_image.height as usize,
        );

        let right_wall_image = sprite_manager.get_image(wall_texture_bindings.right.sprite_id);
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
            let line_width = (line_width_scale * w as f64 / perp_wall_dist) as i32;
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
                            let posi = pos.y as i32;
                            if hide_floor_ceiling && (map_coord.y == posi + 1 || map_coord.y == posi - 1) {
                                None
                            } else {
                                if ray_dir.y > 0.0 {
                                    Some(texture_bindings.floor)
                                } else {
                                    Some(texture_bindings.ceiling)
                                }
                            }
                        }
                        Vertical => Some(texture_bindings.wall), //side
                    }
                }
            };

            if let Some(texture_id) = sid {
                let tex_width_u = sprite_manager.get_image(texture_id).width as usize;
                let tex_height_u = sprite_manager.get_image(texture_id).height as usize;
                let tex_width = sprite_manager.get_image(texture_id).width as f64;
                let tex_height = sprite_manager.get_image(texture_id).height as f64;

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

                let wall_pixels = sprite_manager.get_image(texture_id).get_image_data();

                for x in draw_start..draw_end {
                    let tex_x = (tex_pos as usize).clamp(0, tex_width_u - 1);
                    tex_pos += step;

                    let cvp = wall_pixels[tex_y * tex_height_u + tex_x];
                    let cv = Color::from_rgba(cvp[0], cvp[1], cvp[2], 255).to_vec();

                    let pixel = &mut rd[y * rw + x];
                    *pixel = Color::new(cv.x * fog, cv.y * fog, cv.z * fog, 1.0).into();
                }
            } else {
                for x in draw_start..draw_end {
                    let pixel = &mut rd[y * rw + x];
                    *pixel = Color::new(0.0, 0.0, 0.0, 0.0).into(); // can optimize
                }
            }

            // Draw walls
            if !hide_walls {
                for x in 0..draw_start {
                    let current_dist = line_width_scale * w as f64 / (-2.0 * x as f64 + w as f64); // This can be a table
                    let weight = (current_dist - dist_player) / (dist_wall - dist_player);

                    let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;

                    let (u, v) = if !wall_pin {
                        let uv =
                            apply_boundary_conditions_f64(current_floor_pos, world.grid.get_size());
                        (1.0 - uv.x / world_size.x, uv.y / world_size.y)
                    } else {
                        let distx =
                            (current_floor_pos - pos).dot(DVec2::new(wall_speed * dir_x_sign, 0.0));
                        let disty = (current_floor_pos - pos).dot(DVec2::new(0.0, wall_speed));
                        (
                            wrap_double_norm(distx / max_ray_distance),
                            wrap_double_norm(disty / max_ray_distance),
                        )
                    };

                    // Left wall tex coords
                    let left_tex_x = ((left_wall_width - 1) as f64 * u) as usize;
                    let left_tex_y = ((left_wall_height - 1) as f64 * v) as usize;

                    // Right wall tex coords
                    let right_tex_x = ((right_wall_width - 1) as f64 * u) as usize;
                    let right_tex_y = ((right_wall_height - 1) as f64 * v) as usize;

                    let left_wall_color = left_wall_pixels[left_tex_y * left_wall_width + left_tex_x];
                    let right_wall_color =
                        right_wall_pixels[right_tex_y * right_wall_width + right_tex_x];

                    let wall_fog = fog_factor(current_dist, max_ray_distance) as f32;

                    let mut lc = Color::from_rgba(
                        left_wall_color[0],
                        left_wall_color[1],
                        left_wall_color[2],
                        left_wall_color[3],
                    )
                        .to_vec()
                        * wall_fog;

                    let mut rc = Color::from_rgba(
                        right_wall_color[0],
                        right_wall_color[1],
                        right_wall_color[2],
                        right_wall_color[3],
                    )
                        .to_vec()
                        * wall_fog;

                    rd[y * rw + x] = Color::from_vec(lc).into();
                    rd[y * rw + (render_width as usize - 1 - x)] = Color::from_vec(rc).into();
                }
            }
        }
    }

    pub fn draw_view_horizontal(
        &mut self,
        max_ray_distance: f64,
        world: &Level,
        pos: DVec2,
        dir: DVec2,
        plane_scale: f64,
        front_image: Option<&Self>,
        floor_array: &Vec<Option<SpriteId>>,
        ceiling_array: &Vec<Option<SpriteId>>,
        image_manager: &ImageLoader,
        hide_floor_ceiling: bool,
        lhs: f64,
        wall_texture_bindings: &WallTextureBindings
    ) {
        let plane = plane_scale * dir.perp();
        let (render_width, render_height) = self.render_size;
        let rd = self.render_image.get_image_data_mut();

        let world_size = ucoords_to_dvec2(world.grid.get_size()).as_vec2();

        for x in 0..(render_width as usize) {
            let x_d = x as f64;
            let camera_x = (2.0 * x_d / (render_width as f64) - 1.0);
            let ray_dir = DVec2::from((dir.x + plane.x * camera_x, dir.y + plane.y * camera_x));

            let (perp_wall_dist, hit_type, hit_side, map_coord) =
                cast_ray(&world.grid, &pos, &ray_dir, max_ray_distance);
            let h = render_height as i32;
            let line_height = (lhs * h as f64 / perp_wall_dist) as i32;
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

            // Store z buffer
            match hit_type {
                WallGridCell::Empty => self.z_buffer[x] = f64::INFINITY,
                WallGridCell::Wall => self.z_buffer[x] = perp_wall_dist,
            }

            let dist_wall = perp_wall_dist;
            let dist_player = 0.0f64;

            // Wall is previously rendered frame
            if let Some(wall_view) = front_image {
                let (tex_pixels, tex_width_u, tex_height_u, tex_x, mut tex_pos, step) =
                    {
                        let image = &wall_view.render_image;
                        let tex_width_u = image.width as usize;
                        let tex_height_u = image.height as usize;
                        let tex_width = image.width as f64;
                        let tex_height = image.height as f64;

                        // Calculate textX
                        let mut tex_x = (wall_x * tex_width) as usize;

                        //let mut tex_x = (wall_x * tex_width) as usize;
                        if hit_side == Vertical && dir.x < 0.0 {
                            tex_x = tex_width as usize - tex_x - 1;
                        } // The ifs may need to change
                        if hit_side == Horizontal && dir.y > 0.0 {
                            tex_x = tex_width as usize - tex_x - 1;
                        }

                        // How much to step
                        let step = (tex_height / (line_height as f64));

                        // starting texture pos
                        let tex_pos = (draw_start as i32 - h / 2 + line_height / 2) as f64 * step;
                        let sprite_pixels = image.get_image_data();
                        (sprite_pixels, tex_width_u, tex_height_u, tex_x, tex_pos, step)
                    };

                for y in draw_start..draw_end {
                    let tex_y = (tex_pos as usize).clamp(0, tex_height_u - 1);
                    tex_pos += step;

                    if hit_type != Empty {
                        let cvp = tex_pixels[tex_y * tex_width_u + tex_x];
                        if cvp[3] != 0 {
                            let pixel = &mut rd[y * render_width as usize + x];
                            *pixel = cvp;
                        }
                    }
                }
            } else {
                let line_height_2 = (16.0 * line_height as f64) as i32;
                let n = 4.0;
                let true_draw_start = h/2 - (line_height as f64 *(0.5 + n)) as i32;
                let draw_start_2 = 0.max(true_draw_start);
                let draw_end = h.min(line_height_2 / 2 + h / 2) as usize;

                let image = image_manager.get_image(wall_texture_bindings.left.sprite_id);
                let tex_width_u = image.width as usize;
                let tex_height_u = image.height as usize;
                let tex_width = image.width as f64;
                let tex_height = image.height as f64;

                let wall_hit = perp_wall_dist * ray_dir;

                // Calculate textX
                let mut tex_x = ((perp_wall_dist / max_ray_distance) * tex_width) as usize;
                //let mut tex_x = (wall_x * tex_width) as usize;
                if hit_side == Vertical && dir.x < 0.0 {
                    tex_x = tex_width as usize - tex_x - 1;
                } // The ifs may need to change
                if hit_side == Horizontal && dir.y > 0.0 {
                    tex_x = tex_width as usize - tex_x - 1;
                }

                // How much to step
                let step = (tex_height / (line_height_2 as f64));

                // starting texture pos
                let mut tex_pos = (draw_start_2 - true_draw_start) as f64 * step;
                let tex_pixels = image.get_image_data();


                for y in draw_start_2 as usize..draw_end {
                    let tex_y = (tex_pos as usize).clamp(0, tex_height_u - 1);
                    tex_pos += step;

                    if hit_side == Horizontal{
                        let cvp = tex_pixels[tex_y * tex_width_u + tex_x];
                        let pixel = &mut rd[y * render_width as usize + x];
                        *pixel = cvp.into();
                    } else {
                        let pixel = &mut rd[y * render_width as usize + x];
                        *pixel = BLACK.into();
                    }
                }
            }

            // Draw floor and ceiling
            if !hide_floor_ceiling {
                for y in draw_end + 1..h as usize {
                    let current_dist = lhs * h as f64 / (2.0 * y as f64 - h as f64 + 2.0); // This can be a table
                    let weight = ((current_dist - dist_player) / (dist_wall - dist_player));
                    let current_floor_pos = weight * wall_hit_coord + (1.0 - weight) * pos;

                    let uv = current_floor_pos;
                    let map_x = uv.x as usize;

                    let fog =
                        fog_factor(current_floor_pos.distance(pos), max_ray_distance) as f32;

                    let floor_tile = floor_array[map_x];
                    let floor_color:Option<Color> = match floor_tile {
                        None => None,
                        Some(tex_id) => {
                            let floor_tex = image_manager.get_image(tex_id);
                            let u = 1.0 - uv.y;
                            let v = 1.0 - (uv.x - map_x as f64);

                            let tex_x = (u * (floor_tex.width() - 1) as f64) as u32;
                            let tex_y = (v * (floor_tex.height() - 1) as f64) as u32;

                            let mut cv = floor_tex.get_pixel(tex_x, tex_y).to_vec() * fog;
                            cv.w = 1.0;
                            Some(Color::from_vec(cv))
                        }
                    };

                    let ceiling_tile = ceiling_array[map_x];
                    let ceiling_color:Option<Color> = match ceiling_tile {
                        None => None,
                        Some(tex_id) => {
                            let ceiling_tex = image_manager.get_image(tex_id);
                            let u = 1.0 - uv.y;
                            let v = 1.0 - (uv.x - map_x as f64);

                            let tex_x = (u * (ceiling_tex.width() - 1) as f64) as u32;
                            let tex_y = (v * (ceiling_tex.height() - 1) as f64) as u32;

                            let mut cv = ceiling_tex.get_pixel(tex_x, tex_y).to_vec() * fog;
                            cv.w = 1.0;
                            Some(Color::from_vec(cv))
                        }
                    };

                    if let Some(c) = floor_color {
                        rd[y * render_width as usize + x] = c.into();
                    }
                    if let Some(c) = ceiling_color {
                        rd[(render_height as usize - 1 - y) * render_width as usize + x] = c.into();
                    }
                }
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
