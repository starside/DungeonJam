use macroquad::color::{BLACK, BLUE, Color, DARKGREEN, SKYBLUE, WHITE};
use macroquad::math::{DVec2, Vec2};
use macroquad::miniquad::FilterMode;
use macroquad::prelude::{draw_texture_ex, DrawTextureParams, Image, Texture2D};
use crate::grid2d::GridCellType;
use crate::level::Level;
use crate::raycaster::{cast_ray, HitSide};

pub struct FirstPersonViewer {
    render_size: (u16, u16),
    render_image: Image,
    render_texture: Texture2D
}

impl FirstPersonViewer {
    pub fn new(width: u16, height: u16) -> Self {
        let render_image = Image::gen_image_color(width, height, BLACK);
        let render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);

        FirstPersonViewer {
            render_size: (width, height),
            render_image,
            render_texture
        }
    }
    pub fn draw_view(
        &mut self,
        world: &Level,
        screen_size: (f32, f32),
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

        for y in 0..(render_height as usize) {
            let y_d = y as f64;
            let camera_y =  up*(2.0 * y_d / (render_height as f64) - 1.0);
            let ray_dir_x = dir.x + plane.x * camera_y;
            let ray_dir_y = dir.y + plane.y * camera_y;
            let ray_dir = DVec2::from((ray_dir_x, ray_dir_y));

            let (perp_wall_dist, hit_type, hit_side, _) = cast_ray(&world.grid, &pos, &ray_dir);
            let w = render_width as i32;
            let line_width = (w as f64 / perp_wall_dist) as i32;
            let draw_start = 0.max((-line_width/2) + (w/2)) as usize;
            let draw_end = w.min(line_width / 2 + w / 2) as usize;
            let rw = render_width as usize;

            let fog = f64::exp(-(perp_wall_dist/10.0).powi(2)) as f32;
            let color =
                match hit_type {
                    GridCellType::Empty => { BLACK }
                    GridCellType::Wall => {
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

            for x in 0..draw_start {
                rd[y * rw + x] = BLACK.into()
            }

            for x in draw_start..draw_end {
                let cv = Color::to_vec(&color);
                let pixel = &mut rd[y * rw + x];
                *pixel = Color::from_vec(fog * cv).into();
            }

            for x in draw_end..render_width as usize {
                rd[y * rw + x] = BLACK.into()
            }
        }

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