mod boid;
mod node;
mod background;

use pixels::{self, Pixels, SurfaceTexture};
use winit::{self, dpi::{PhysicalSize}, event::{ElementState, Event, MouseButton, WindowEvent}, event_loop::{EventLoop}, window::WindowBuilder};
use rand::Rng;
use winit::dpi::PhysicalPosition;
use boid::Boid;
use node::{ RenderNode, MovableMode };
use background::Background;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SIZE: i16 = 10;
const AVOID_FACTOR: f32 = 0.1;
const MATCHING_FACTOR: f32 = 0.25;
const CENTERING_FACTOR: f32 = 0.25;
const SAFE_RADIUS: f32 = 50.0;
const MAX_SPEED: i16 = 25;
const MIN_SPEED: i16 = 5;
const NUMBER_OF_BOIDS: u16 = 300;

const MARGIN: i16 = 100;
const TURN_FACTOR: i16 = 8;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = {
        let size = PhysicalSize::new(WIDTH, HEIGHT);
        WindowBuilder::new()
            .with_title("Boids")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let mut world = World::new();
    let mut mouse_press: bool = false;
    let mut mouse_position: PhysicalPosition<f64> = PhysicalPosition::new(0.0, 0.0);

    world.spawn_random_boids(NUMBER_OF_BOIDS);

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                println!("Close Window");
                elwt.exit();
            },
            Event::AboutToWait => {
                world.update();
                window.request_redraw();
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, ..} => {
                world.draw(pixels.frame_mut());
                if let Err(error) = pixels.render() {
                    elwt.exit();
                    eprint!("{error}");
                }
            },
            Event::WindowEvent { event: WindowEvent::MouseInput{ button, state, .. }, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed && mouse_press == false {
                    mouse_press = true;
                    println!("{:}, {:}", mouse_position.x, mouse_position.y);
                    world.spawn_boids(mouse_position.x as i16, mouse_position.y as i16);
                }
                if button == MouseButton::Left && state == ElementState::Released && mouse_press == true {
                    mouse_press = false;
                }
            },
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                mouse_position = position;
            }
            _ => ()
        }
    });
}

struct World {
    background: Background,
    boids: Vec<Boid>
}

impl World {
    fn new() -> Self {
        Self {
           background: Background::new([0, 0, 0, 0]),
           boids: Vec::new(),
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
        let velocity_x = rng.gen_range(-MIN_SPEED..=MIN_SPEED);
        let range: [i16; 2] = [-1, 1];
        let velocity_y = ((MIN_SPEED.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16 * range[rng.gen_range(0..=1)];
        self.boids.push(Boid::new(x, y, SIZE, velocity_x, velocity_y, [255, 255, 255, 255]));
    }

    fn draw(&self, frame: &mut [u8]) {
        self.background.draw(frame, WIDTH, HEIGHT);
        for boid in &self.boids {
            boid.draw(frame, WIDTH, HEIGHT)
        }
    }

    fn update(&mut self) {
        let copy_boids: Vec<Boid> = self.boids.to_vec();
        for boid in &mut self.boids {
            boid.separate(&copy_boids, AVOID_FACTOR, SAFE_RADIUS);
            boid.align(&copy_boids, MATCHING_FACTOR, SAFE_RADIUS);
            boid.cohesion(&copy_boids, CENTERING_FACTOR, SAFE_RADIUS);
            boid.avoid_border(TURN_FACTOR, MARGIN, WIDTH, HEIGHT);
            boid.speed_limit(MAX_SPEED, MIN_SPEED);
            boid.update(WIDTH, HEIGHT, SIZE);
        }
    }
}
