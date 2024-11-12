mod background;
mod boid;
mod gui;
mod node;

use core::panic;
use std::fmt::Display;

use background::Background;
use boid::Boid;
use gui::Framework;
use node::{MovableMode, RenderNode, Vertice};
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

const WIDTH: u16 = 1920;
const HEIGHT: u16 = 1080;
const SIZE: i16 = 3;
const NUMBER_OF_BOIDS: u16 = 2000;
const QUAD_TREE_CAPACITY: usize = 4;

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

struct World {
    background: Background,
    quad_tree: QuadTree,
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
            quad_tree: QuadTree::new(QUAD_TREE_CAPACITY, Rectangle::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0)),
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
            let x = rng.gen_range(0..WIDTH - SIZE as u16) as i16;
            let y = rng.gen_range(0..HEIGHT - SIZE as u16) as i16;

            self.spawn_boids(x, y);
        }
    }

    fn spawn_boids(&mut self, x: i16, y: i16) {
        let mut rng = rand::thread_rng();
        let velocity_x = rng.gen_range(-self.min_speed..=self.min_speed);
        let range: [i16; 2] = [-1, 1];
        let velocity_y = ((self.min_speed.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16
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
        self.quad_tree = QuadTree::new(QUAD_TREE_CAPACITY, Rectangle::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0));
    }

    fn draw(&self, frame: &mut [u8]) {
        self.background.draw(frame, WIDTH, HEIGHT);
        self.quad_tree.draw(frame);
    }

    fn update(&mut self) {
        let mut new_quard_tree = QuadTree::new(QUAD_TREE_CAPACITY, Rectangle::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0, WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0));
        for boid in self.quad_tree.to_vec() {
            let mut new_boid = boid.clone();
            let mut found: Vec<Boid> = vec![];
            self.quad_tree.query(&mut found, &boid, self.vision_radius);
            new_boid.separate(
                &found,
                self.avoid_factor,
                self.safe_radius,
                self.view_angle,
            );
            new_boid.align(
                &found,
                self.matching_factor,
                self.vision_radius,
                self.view_angle,
            );
            new_boid.cohesion(
                &found,
                self.centering_factor,
                self.vision_radius,
                self.view_angle,
            );
            new_boid.avoid_border(self.turn_factor, self.margin, WIDTH, HEIGHT);
            new_boid.noise(self.noise);
            new_boid.speed_limit(self.max_speed, self.min_speed);
            new_boid.update_color(self.max_speed, self.min_speed);
            new_boid.update(WIDTH, HEIGHT);
            new_quard_tree.insert(&new_boid);
        }
        self.quad_tree = new_quard_tree.clone();
    }
}

#[derive(Clone)]
struct Rectangle {
    center_x: f32,
    center_y: f32,
    half_width: f32,
    half_height: f32,
}

impl Rectangle {
    fn new(center_x: f32, center_y: f32, half_width: f32, half_height: f32) -> Self {
        Self {
            center_x,
            center_y,
            half_width,
            half_height,
        }
    }

    fn contains_point(&self, x: f32, y: f32) -> bool {
        let min_x = self.center_x - self.half_width;
        let max_x = self.center_x + self.half_width;
        let min_y = self.center_y - self.half_height;
        let max_y = self.center_y + self.half_height;
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    fn intersects(&self, vertice: &Vertice, vision_radius: f32) -> bool {
        let dx = self.center_x - vertice.x as f32;
        let dy = self.center_y - vertice.y as f32;
        let d = (dx * dx + dy * dy).sqrt();
        d <= vision_radius
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}, width: {}, height: {}", self.center_x, self.center_y, self.half_width, self.half_height)
    }
}

#[derive(Clone)]
struct QuadTree {
    capacity: usize,
    boundary: Rectangle,
    boids: Vec<Boid>,
    top_right: Option<Box<QuadTree>>,
    top_left: Option<Box<QuadTree>>,
    bottom_right: Option<Box<QuadTree>>,
    bottom_left: Option<Box<QuadTree>>,
    splitted: bool,
}

impl QuadTree {
    fn new(capacity: usize, boundary: Rectangle) -> Self {
        Self {
            capacity,
            boundary,
            boids: vec![],
            top_right: None,
            top_left: None,
            bottom_right: None,
            bottom_left: None,
            splitted: false,
        }
    }

    fn insert(&mut self, boid: &Boid) -> bool {
        if !self.boundary.contains_point(boid.vertice.x as f32, boid.vertice.y as f32) {
            return false;
        }
        if self.boids.len() < self.capacity {
            self.boids.push(boid.clone());
            return true;
        }
        if !self.splitted {
            self.split();
            self.splitted = true;
        }
        match &mut self.top_left {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            },
            None => {
                panic!("Top left is not create");
            }
        }
        match &mut self.top_right {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            },
            None => {
                panic!("Top right is not create");
            }
        }
        match &mut self.bottom_left {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            },
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &mut self.bottom_right {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            },
            None => {
                panic!("Bottom right is not create");
            }
        }
        println!("{}", boid.vertice);
        false
    }

    fn split(&mut self) {
        let tr = Rectangle::new(
            self.boundary.center_x + self.boundary.half_width / 2.0,
            self.boundary.center_y + self.boundary.half_height / 2.0,
            self.boundary.half_width / 2.0,
            self.boundary.half_height / 2.0,
        );
        self.top_right = Some(Box::new(QuadTree::new(self.capacity, tr)));
        let tl = Rectangle::new(
            self.boundary.center_x - self.boundary.half_width / 2.0,
            self.boundary.center_y + self.boundary.half_height / 2.0,
            self.boundary.half_width / 2.0,
            self.boundary.half_height / 2.0,
        );
        self.top_left = Some(Box::new(QuadTree::new(self.capacity, tl)));
        let br = Rectangle::new(
            self.boundary.center_x + self.boundary.half_width / 2.0,
            self.boundary.center_y - self.boundary.half_height / 2.0,
            self.boundary.half_width / 2.0,
            self.boundary.half_height / 2.0,
        );
        self.bottom_right = Some(Box::new(QuadTree::new(self.capacity, br)));
        let bl = Rectangle::new(
            self.boundary.center_x - self.boundary.half_width / 2.0,
            self.boundary.center_y - self.boundary.half_height / 2.0,
            self.boundary.half_width / 2.0,
            self.boundary.half_height / 2.0,
        );
        self.bottom_left = Some(Box::new(QuadTree::new(self.capacity, bl)));
    }

    fn query(&self, found: &mut Vec<Boid>, boid: &Boid, vision_radius: f32) {
        if !self.boundary.intersects(&boid.vertice, vision_radius) && !self.boundary.contains_point(boid.vertice.x as f32, boid.vertice.y as f32) {
            return;
        }
        for other_boid in &self.boids {
            if other_boid != boid {
                found.push(other_boid.clone());
            }
        }
        if !self.splitted {
            return;
        }
        match &self.top_left {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            },
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            },
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            },
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            },
            None => {
                panic!("Bottom right is not create");
            }
        }
    }

    fn draw(&self, frame: &mut [u8]) {
        for boid in &self.boids {
            boid.draw(frame, WIDTH, HEIGHT);
        }
        if !self.splitted {
            return;
        }
        match &self.top_left {
            Some(q_tree) => {
                q_tree.draw(frame);
            },
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.draw(frame);
            },
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.draw(frame);
            },
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.draw(frame);
            },
            None => {
                panic!("Bottom right is not create");
            }
        }
    }

    fn to_vec(&self) -> Vec<Boid> {
        let mut boids: Vec<Boid> = vec![];
        for boid in &self.boids {
            boids.push(boid.clone());
        }
        if !self.splitted {
            return boids;
        }
        match &self.top_left {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            },
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            },
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            },
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            },
            None => {
                panic!("Bottom right is not create");
            }
        }
        boids
    }
}
