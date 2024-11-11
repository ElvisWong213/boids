mod boid;
mod node;
mod background;
mod gui;

use pixels::{self, Pixels, SurfaceTexture};
use winit::{self, dpi::PhysicalSize, event::{ElementState, Event, MouseButton, WindowEvent}, event_loop::EventLoop, window::WindowBuilder};
use rand::Rng;
use winit::dpi::PhysicalPosition;
use boid::Boid;
use node::{ MovableMode, RenderNode, Vertice };
use background::Background;
use gui::Framework;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SIZE: i16 = 3;
const NUMBER_OF_BOIDS: u16 = 200;

fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = PhysicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Boids")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let (mut pixels, mut framework) = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();
        let framework = Framework::new(&event_loop, window_size.width, window_size.height, scale_factor, &pixels);

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
            },
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
            },
            Event::WindowEvent { event, .. } => {
                // Update egui inputs
                let event_response = framework.handle_event(&event);
                if !event_response.consumed {
                    match event {
                        WindowEvent::CloseRequested => {
                            println!("Close Window");
                            elwt.set_exit();
                        },
                        WindowEvent::MouseInput{ button, state, .. } => {
                            if button == MouseButton::Left && state == ElementState::Pressed && !mouse_press {
                                mouse_press = true;
                                println!("{:}, {:}", mouse_position.x, mouse_position.y);
                                world.spawn_boids(mouse_position.x as i16, mouse_position.y as i16);
                            }
                            if button == MouseButton::Left && state == ElementState::Released && mouse_press {
                                mouse_press = false;
                            }
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            mouse_position = position;
                        },
                        WindowEvent::Resized(new_size) => {
                            if new_size.width > 0 && new_size.height > 0 {
                                pixels.resize_surface(new_size.width, new_size.height).unwrap();
                            }
                            framework.resize(new_size.width, new_size.height);
                        },
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            framework.scale_factor(scale_factor);
                        }
                        _ => ()
                    }
                };
            },
            _ => ()
        }
    });
}

struct World {
    background: Background,
    boids: Vec<Boid>,
    avoid_factor: f32,
    matching_factor: f32,
    centering_factor: f32,
    safe_radius: f32,
    vision_radius: f32,
    max_speed: i16,
    min_speed: i16,
    margin: i16,
    turn_factor: i16,
    noise: bool,
    view_angle: f32,
}

impl World {
    fn new() -> Self {
        Self {
            background: Background::new([0, 0, 0, 0]),
            boids: Vec::new(),
            avoid_factor: 0.27,
            matching_factor: 0.55,
            centering_factor: 0.06,
            safe_radius: 30.0,
            vision_radius: 80.0,
            max_speed: 10,
            min_speed: 5,
            margin: 20,
            turn_factor: 30,
            noise: false,
            view_angle: 120.0,
        }
    }

    fn spawn_random_boids(&mut self, numbers: u16) {
        let mut rng = rand::thread_rng();
        for _ in 0..numbers {
            let x = rng.gen_range(0..WIDTH - SIZE as u32) as i16;
            let y = rng.gen_range(0..HEIGHT - SIZE as u32) as i16;

            self.spawn_boids(x, y); 
        }
    }

    fn spawn_boids(&mut self, x: i16, y:i16) {
        let mut rng = rand::thread_rng();
        let velocity_x = rng.gen_range(-self.min_speed..=self.min_speed);
        let range: [i16; 2] = [-1, 1];
        let velocity_y = ((self.min_speed.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16 * range[rng.gen_range(0..=1)];
        let mut vertice = Vertice::new();
        vertice.x = x;
        vertice.y = y;
        self.boids.push(Boid::new(vertice, SIZE, velocity_x, velocity_y, [255, 255, 0, 255]));
    }

    fn restart(&mut self) {
        self.clear_all();
        self.spawn_random_boids(NUMBER_OF_BOIDS);
    }

    fn clear_all(&mut self) {
        self.boids = vec![]
    }

    fn draw(&self, frame: &mut [u8]) {
        self.background.draw(frame, WIDTH, HEIGHT);
        for boid in &self.boids {
            boid.draw(frame, WIDTH, HEIGHT);
        }
    }

    fn update(&mut self) {
        let copy_boids: Vec<Boid> = self.boids.to_vec();
        for boid in &mut self.boids {
            boid.separate(&copy_boids, self.avoid_factor, self.safe_radius, self.view_angle);
            boid.align(&copy_boids, self.matching_factor, self.vision_radius, self.view_angle);
            boid.cohesion(&copy_boids, self.centering_factor, self.vision_radius, self.view_angle);
            boid.avoid_border(self.turn_factor, self.margin, WIDTH, HEIGHT);
            boid.noise(&self.noise);
            boid.speed_limit(self.max_speed, self.min_speed);
            boid.update(WIDTH, HEIGHT);
            boid.update_color(&self.max_speed, &self.min_speed);
        }
    }
}
