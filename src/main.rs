mod background;
mod boid;
mod gui;
mod node;
mod geometry;

use std::time::SystemTime;

use background::Background;
use boid::Boid;
use geometry::{Color, Rectangle};
use gui::Framework;
use node::{MovableNode, QuadTree, RenderNode, Vertice};
use pixels::{self, Pixels, SurfaceTexture};
use rand::Rng;
use winit::dpi::PhysicalPosition;
use winit::{
    self,
    dpi::PhysicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const WIDTH: u16 = 1280;
const HEIGHT: u16 = 720;
const SIZE: i16 = 3;
const NUMBER_OF_BOIDS: u16 = 1000;
const QUAD_TREE_CAPACITY: usize = 4;

fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = PhysicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Boids")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap();
        let framework = Framework::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &pixels,
        );

        (pixels, framework)
    };

    let mut world = World::new();
    let mut mouse_press: bool = false;
    let mut mouse_position: PhysicalPosition<f64> = PhysicalPosition::new(0.0, 0.0);

    world.spawn_random_boids(NUMBER_OF_BOIDS);

    event_loop.run(move |event, _, elwt| {
        match event {
            Event::MainEventsCleared => {
                framework.prepare(&window, &mut world);
                world.update();
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                world.draw(pixels.frame_mut());
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    context.scaling_renderer.render(encoder, render_target);
                    framework.render(encoder, render_target, context);
                    Ok(())
                });

                if let Err(error) = render_result {
                    elwt.set_exit();
                    eprint!("{error}");
                }
            }
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                let event_response = framework.handle_event(&event);
                if !event_response.consumed {
                    match event {
                        WindowEvent::CloseRequested => {
                            println!("Close Window");
                            elwt.set_exit();
                        }
                        WindowEvent::MouseInput { button, state, .. } => {
                            if button == MouseButton::Left
                                && state == ElementState::Pressed
                                && !mouse_press
                            {
                                mouse_press = true;
                                println!("{:}, {:}", mouse_position.x, mouse_position.y);
                                world.spawn_boids(mouse_position.x as i16, mouse_position.y as i16);
                            }
                            if button == MouseButton::Left
                                && state == ElementState::Released
                                && mouse_press
                            {
                                mouse_press = false;
                            }
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            mouse_position = position;
                        }
                        WindowEvent::Resized(new_size) => {
                            if new_size.width > 0 && new_size.height > 0 {
                                pixels
                                    .resize_surface(new_size.width, new_size.height)
                                    .unwrap();
                            }
                            framework.resize(new_size.width, new_size.height);
                        }
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            framework.scale_factor(scale_factor);
                        }
                        _ => (),
                    }
                };
            }
            _ => (),
        }
    });
}

struct WorldOption {
    avoid_factor: f32,
    matching_factor: f32,
    centering_factor: f32,
    safe_radius: f32,
    vision_radius: f32,
    max_speed: i16,
    min_speed: i16,
    margin: u16,
    turn_factor: i16,
    view_angle: f32,
    noise: bool,
    show_quad_tree: bool,
    show_safe_radius: bool,
    show_vision_radius: bool,
    show_facing_direction_with_speed: bool,
}

impl WorldOption {
    fn new() -> Self {
        Self {
            avoid_factor: 0.27,
            matching_factor: 0.55,
            centering_factor: 0.06,
            safe_radius: 30.0,
            vision_radius: 80.0,
            max_speed: 10,
            min_speed: 5,
            margin: 20,
            turn_factor: 30,
            view_angle: 120.0,
            noise: false,
            show_quad_tree: false,
            show_safe_radius: false,
            show_vision_radius: false,
            show_facing_direction_with_speed: false,
        }
    }
}

struct World {
    background: Background,
    boundary: Rectangle,
    quad_tree: QuadTree,
    update_fps: f32,
    draw_fps: f32,
    option: WorldOption,
}

impl World {
    fn new() -> Self {
        Self {
            background: Background::new(Color::Black),
            boundary: Rectangle::new(
                WIDTH as f32 / 2.0,
                HEIGHT as f32 / 2.0,
                WIDTH as f32 / 2.0,
                HEIGHT as f32 / 2.0,
            ),
            quad_tree: QuadTree::new(
                QUAD_TREE_CAPACITY,
                Rectangle::new(
                    WIDTH as f32 / 2.0,
                    HEIGHT as f32 / 2.0,
                    WIDTH as f32 / 2.0,
                    HEIGHT as f32 / 2.0,
                ),
            ),
            update_fps: 0.0,
            draw_fps: 0.0,
            option: WorldOption::new(),
        }
    }

    fn spawn_random_boids(&mut self, numbers: u16) {
        let mut rng = rand::thread_rng();
        for _ in 0..numbers {
            let x = rng.gen_range(0..WIDTH - SIZE as u16) as i16;
            let y = rng.gen_range(0..HEIGHT - SIZE as u16) as i16;

            self.spawn_boids(x, y);
        }
    }

    fn spawn_boids(&mut self, x: i16, y: i16) {
        let mut rng = rand::thread_rng();
        let velocity_x = rng.gen_range(-self.option.min_speed..=self.option.min_speed);
        let range: [i16; 2] = [-1, 1];
        let velocity_y = ((self.option.min_speed.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16
            * range[rng.gen_range(0..=1)];
        let mut vertice = Vertice::new();
        vertice.x = x;
        vertice.y = y;
        self.quad_tree.insert(&Boid::new(
            vertice,
            SIZE,
            velocity_x,
            velocity_y,
            [255, 255, 0, 255],
        ));
    }

    fn restart(&mut self) {
        self.clear_all();
        self.spawn_random_boids(NUMBER_OF_BOIDS);
    }

    fn clear_all(&mut self) {
        self.quad_tree = QuadTree::new(QUAD_TREE_CAPACITY, self.boundary.clone());
    }

    fn draw(&mut self, frame: &mut [u8]) {
        let start_time = SystemTime::now();
        self.background.draw(frame, WIDTH, HEIGHT);
        self.quad_tree.draw_with_option(frame, WIDTH, HEIGHT, &self.option);
        let end_time = SystemTime::now();
        Self::update_fps_count(&mut self.draw_fps, start_time, end_time);
    }

    fn update(&mut self) {
        let start_time = SystemTime::now();
        let mut new_quard_tree = QuadTree::new(QUAD_TREE_CAPACITY, self.boundary.clone());
        for boid in self.quad_tree.to_vec() {
            let mut new_boid = boid.clone();
            let mut found: Vec<Boid> = vec![];
            self.quad_tree.query(&mut found, &boid, self.option.vision_radius);
            new_boid.separate(&found, self.option.avoid_factor, self.option.safe_radius, self.option.view_angle);
            new_boid.align(
                &found,
                self.option.matching_factor,
                self.option.vision_radius,
                self.option.view_angle,
            );
            new_boid.cohesion(
                &found,
                self.option.centering_factor,
                self.option.vision_radius,
                self.option.view_angle,
            );
            new_boid.noise(self.option.noise);
            new_boid.speed_limit(self.option.max_speed, self.option.min_speed);
            new_boid.avoid_border(self.option.turn_factor, self.option.margin, WIDTH, HEIGHT);
            new_boid.update(WIDTH, HEIGHT);
            new_boid.update_color(self.option.max_speed, self.option.min_speed);
            new_quard_tree.insert(&new_boid);
        }
        self.quad_tree = new_quard_tree.clone();
        let end_time = SystemTime::now();
        Self::update_fps_count(&mut self.update_fps, start_time, end_time);
    }

    fn update_fps_count(fps: &mut f32, start_time: SystemTime, end_time: SystemTime) {
        match end_time.duration_since(start_time) {
            Ok(duration) => {
                *fps = 1.0 / duration.as_secs_f32();
            }
            Err(_) => {
                println!("Cannot get duration");
            }
        }
    }
}
