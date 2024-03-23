mod raycaster;
mod grid2d;
mod grid_viewer;
use macroquad::miniquad::window::screen_size;
use macroquad::prelude::*;
use crate::grid2d::GridCellType;
use crate::raycaster::{cast_ray, HitSide};

#[derive(Default)]
struct DebugView {
    debug_line: (Vec2, Vec2)
}
struct FirstPersonViewer {
    render_size: (u16, u16),
    render_image: Image,
    render_texture: Texture2D
}

impl FirstPersonViewer {
    fn new(width: u16, height: u16) -> Self {
        let render_image = Image::gen_image_color(width, height, BLACK);
        let render_texture = Texture2D::from_image(&render_image);
        render_texture.set_filter(FilterMode::Nearest);

        FirstPersonViewer {
            render_size: (width, height),
            render_image,
            render_texture
        }
    }
    fn draw_view(
        &mut self,
        world: &grid2d::Grid2D<grid2d::RayGridCell>,
        screen_size: (f32, f32), pos: DVec2, dir: DVec2, plane: DVec2) {
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


            let (perp_wall_dist, hit_type, hit_side, _) = cast_ray(&world, &pos, &ray_dir);
            let w = render_width as i32;
            let line_width = (w as f64 / perp_wall_dist) as i32;
            let draw_start = 0.max((-line_width/2) + (w/2)) as usize;
            let draw_end = w.min(line_width / 2 + w / 2) as usize;
            let rw = render_width as usize;

            for x in 0..draw_start {
                rd[y * rw + x] = BLACK.into()
            }

            let fog = f64::exp(-(perp_wall_dist/8.0).powi(2)) as f32;
            for x in draw_start..draw_end {
                let color =
                    match hit_type {
                        GridCellType::Empty => { DARKPURPLE }
                        GridCellType::Wall => {
                            match hit_side {
                                HitSide::Horizontal => {
                                    if dir.y > 0.0 {
                                        SKYBLUE //top
                                    } else {
                                        DARKGREEN // bottom
                                    }
                                }
                                HitSide::Vertical => { BLUE } //side
                            }
                        }
                    };
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

impl DebugView {
    // Returns position and ray_direction (not normalized), both in world coordinates
    fn draw_debug_view(&mut self, world: &mut grid2d::Grid2D<grid2d::RayGridCell>, screen_size: (f32, f32)) ->
                                                                                                         Option<(DVec2, DVec2)>{
        clear_background(BLACK);
        grid_viewer::draw_grid2d(&world, screen_size);
        match get_last_key_pressed() {
            None => {}
            Some(x) => {
                match &x {
                    KeyCode::S => {
                        self.debug_line.0 = mouse_position().into();
                    },
                    KeyCode::E => {
                        self.debug_line.1 = mouse_position().into();
                    },
                    KeyCode::Q => {
                        return None;
                    }
                    KeyCode::P => {
                        let debug_file = "debug_level.json";
                        world.save_to_file(debug_file);
                        println!("Saved world to {}", debug_file);
                    }
                    KeyCode::L => {
                        let debug_file = "debug_level.json";
                        world.load_from_file(debug_file);
                        println!("Loaded world from {}", debug_file);
                    }
                    _ => {}
                }
            }
        }

        draw_line(self.debug_line.0.x, self.debug_line.0.y,self.debug_line.1.x, self.debug_line.1.y, 1.0, BLUE);
        draw_circle(self.debug_line.0.x, self.debug_line.0.y, 7.0, BLUE);

        let ray_dir = world.screen_to_grid_coords((self.debug_line.1 - self.debug_line.0).as_dvec2(), screen_size);
        let ray_start = world.screen_to_grid_coords(self.debug_line.0.as_dvec2(), screen_size);

        let (perp_hit_dist, _, _, _) = cast_ray(&world, &ray_start, &ray_dir);

        let first_step = world.grid_to_screen_coords(ray_start + perp_hit_dist*ray_dir, screen_size).as_vec2();

        draw_circle(first_step.x, first_step.y, 2.0, RED);

        Some((ray_start, ray_dir))
    }
}

enum GameState {
    Debug,
    FirstPerson
}

#[macroquad::main("BasicShapes")]
async fn main() {
    let (world_width, world_height):(usize, usize) = (16, 16);
    let mut world: grid2d::Grid2D<grid2d::RayGridCell> = grid2d::Grid2D::new(world_width, world_height);
    world.randomize();

    let mut debug_view = DebugView::default();
    let mut pos = DVec2::from((0.0, 0.0));
    let mut dir = DVec2::from((-1.0, 0.0));
    let plane_scale = -0.66;
    let mut plane = plane_scale*dir.perp();

    // Set up low resolution renderer
    let mut first_person_view = FirstPersonViewer::new(640, 480);

    let mut game_state = GameState::Debug;

    loop {
        let size_screen = screen_size();

        match game_state {
            GameState::Debug => {
                if let Some((p, d)) = debug_view.draw_debug_view(&mut world, size_screen) {
                    pos = p;
                    dir = d.normalize();
                    plane = dir.perp() * plane_scale;
                } else {
                    game_state = GameState::FirstPerson;
                }
            }
            GameState::FirstPerson => {
                clear_background(BLACK);
                first_person_view.draw_view(&world, size_screen, pos, dir, plane);
                // Draw FPS
                let fps = get_fps();
                draw_text(format!("{}", fps).as_str(), 20.0, 20.0, 30.0, DARKGRAY);
                match get_last_key_pressed() {
                    None => {}
                    Some(x) => {
                        match &x {
                            KeyCode::Q => {
                                game_state = GameState::Debug;
                            }
                            _ => {}
                        }
                    }
                }

            }
        }

        next_frame().await
    }
}