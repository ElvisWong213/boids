use pixels::{self, Pixels, SurfaceTexture};
use winit::{self, dpi::{PhysicalSize}, event::{ElementState, Event, MouseButton, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use rand::Rng;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SIZE: i16 = 5;
const AVOID_FACTOR: f32 = 0.1;
const MATCHING_FACTOR: f32 = 0.25;
const CENTERING_FACTOR: f32 = 0.25;
const SAFE_RADIUS: f32 = 50.0;
const MAX_SPEED: i16 = 25;
const MIN_SPEED: i16 = 5;
const NUMBER_OF_BOIDS: u16 = 200;

const LEFT_MARGIN: i16 = 200;
const RIGHT_MARGIN: i16 = WIDTH as i16 - 200;
const TOP_MARGIN: i16 = 100;
const BOTTOM_MARGIN: i16 = HEIGHT as i16 - 100;
const TURN_FACTOR: i16 = 3;

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

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

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
                if mouse_press == true {
                    mouse_press = false;
                }
                if button == MouseButton::Left && state == ElementState::Pressed && mouse_press == false {
                    mouse_press = true;
                }
            },
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                if mouse_press == true {
                    println!("{:}, {:}", position.x, position.y);
                    world.spawn_boids(position.x as i16, position.y as i16);
                    mouse_press = false;
                }
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
}

impl RenderNode for World {
   fn draw(&self, frame: &mut [u8]) {
       self.background.draw(frame);
       for boid in &self.boids {
           boid.draw(frame)
       }
   }
}

impl MovableMode for World {
    fn update(&mut self) {
        let copy_boids: Vec<Boid> = self.boids.to_vec();
        for boid in &mut self.boids {
            boid.separate(&copy_boids);
            boid.align(&copy_boids);
            boid.cohesion(&copy_boids);
            boid.avoid_border();
            boid.speed_limit();
            boid.update();
        }
    }
}

trait RenderNode {
    fn draw(&self, _frame: &mut[u8]) {}
}
 trait MovableMode {
     fn update(&mut self) {}
 }

#[derive(Clone, PartialEq)]
struct Boid {
    x: i16,
    y: i16,
    size: i16,
    velocity_x: i16,
    velocity_y: i16,
    color: [u8; 4],
}

impl Boid {
    fn new(x: i16, y: i16, size: i16, velocity_x: i16, velocity_y: i16, color: [u8; 4]) -> Self {
        Self {
            x, 
            y, 
            size,
            velocity_x,
            velocity_y,
            color,
        }
    }

    fn separate(&mut self, boids: &Vec<Boid>) {
        let mut close_dx: f32 = 0.0;
        let mut close_dy: f32 = 0.0;
        let boid_radius: f32 = Boid::radius(self.velocity_x, self.velocity_y);
        for other_boid in boids {
            if self == other_boid {
                continue;
            }
            
            let dx = (self.x - other_boid.x) as f32;
            let dy = (self.y - other_boid.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let other_boid_radius: f32 = Boid::radius(other_boid.velocity_x, other_boid.velocity_y);
            if d <= SAFE_RADIUS && Boid::in_range(boid_radius, other_boid_radius) {
                // let diff: f32 = 1.0 / d;
                close_dx += dx;
                close_dy += dy;
            }
        }
        self.velocity_x += (close_dx * AVOID_FACTOR) as i16;
        self.velocity_y += (close_dy * AVOID_FACTOR) as i16;
    } 

    fn align(&mut self, boids: &Vec<Boid>) {
        let mut neighboring_boids: u16 = 0;
        let mut vx_avg: f32 = 0.0;
        let mut vy_avg: f32 = 0.0;
        let boid_radius: f32 = Boid::radius(self.velocity_x, self.velocity_y);
        for other_boid in boids {
            if self == other_boid {
                continue;
            }
            let dx = (self.x - other_boid.x) as f32;
            let dy = (self.y - other_boid.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let other_boid_radius: f32 = Boid::radius(other_boid.velocity_x, other_boid.velocity_y);
            if d <= SAFE_RADIUS * 1.5 && Boid::in_range(boid_radius, other_boid_radius) {
                vx_avg += other_boid.velocity_x as f32;
                vy_avg += other_boid.velocity_y as f32;
                neighboring_boids += 1;
            }
        }
        if neighboring_boids > 0 {
            vx_avg /= neighboring_boids as f32;
            vy_avg /= neighboring_boids as f32;
            self.velocity_x += (vx_avg * MATCHING_FACTOR) as i16;
            self.velocity_y += (vy_avg * MATCHING_FACTOR) as i16;
        }
    }

    fn cohesion(&mut self, boids: &Vec<Boid>) {
        let mut neighboring_boids: u16 = 0;
        let mut x_avg: f32 = 0.0;
        let mut y_avg: f32 = 0.0;
        let boid_radius: f32 = Boid::radius(self.velocity_x, self.velocity_y);
        for other_boid in boids {
            if self == other_boid {
                continue;
            }
            let dx = (self.x - other_boid.x) as f32;
            let dy = (self.y - other_boid.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let other_boid_radius: f32 = Boid::radius(other_boid.velocity_x, other_boid.velocity_y);
            if d <= SAFE_RADIUS * 1.5 && Boid::in_range(boid_radius, other_boid_radius) {
                x_avg += other_boid.x as f32;
                y_avg += other_boid.y as f32;
                neighboring_boids += 1;
            }
        }
        if neighboring_boids > 0 {
            x_avg /= neighboring_boids as f32;
            y_avg /= neighboring_boids as f32;
            self.velocity_x += ((x_avg - self.x as f32) * CENTERING_FACTOR) as i16;
            self.velocity_y += ((y_avg - self.y as f32) * CENTERING_FACTOR) as i16;
        }
        
    }

    fn avoid_border(&mut self) {
        if self.x < LEFT_MARGIN {
            self.velocity_x += TURN_FACTOR;
        }
        if self.x > RIGHT_MARGIN {
            self.velocity_x -= TURN_FACTOR;
        }
        if self.y < TOP_MARGIN {
            self.velocity_y += TURN_FACTOR;
        }
        if self.y > BOTTOM_MARGIN {
            self.velocity_y -= TURN_FACTOR;
        }
    }

    fn radius(vx: i16, vy: i16) -> f32 {
        let rad = (vy as f32 / vx as f32).atan();
        if vx >= 0 && vy >= 0 {
            return rad;
        }
        if (vx < 0 && vy >= 0) || (vx < 0 && vy < 0) {
            return 1.0 + rad;
        }
        2.0 + rad
    }

    fn in_range(base: f32, target: f32) -> bool {
        let min = base + 0.75;
        let mut max = base - 0.75;
        if max < 0.0 {
            max += 2.0;
        }
        if target <= min || target >= max {
            return true;
        }
        false
    }

    fn speed_limit(&mut self) {
        let speed = ((self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y) as f32).sqrt();
        if speed == 0.0 {
            let mut rng = rand::thread_rng();
            let velocity_x = rng.gen_range(-MIN_SPEED..=MIN_SPEED);
            let range: [i16; 2] = [-1, 1];
            let velocity_y = ((MIN_SPEED.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16 * range[rng.gen_range(0..=1)];
 
            self.velocity_x = velocity_x;
            self.velocity_y = velocity_y;
            return;
        }
        if (speed as i16) > MAX_SPEED {
            self.velocity_x = ((self.velocity_x as f32 / speed) * MAX_SPEED as f32) as i16;
            self.velocity_y = ((self.velocity_y as f32 / speed) * MAX_SPEED as f32) as i16;
        } 
        if (speed as i16) < MIN_SPEED {
            self.velocity_x = ((self.velocity_x as f32 / speed) * MIN_SPEED as f32) as i16;
            self.velocity_y = ((self.velocity_y as f32 / speed) * MIN_SPEED as f32) as i16;
        }
    }
}

impl RenderNode for Boid {
    fn draw(&self, frame: &mut[u8]) {
        for i in 0..self.size {
            for j in 0..self.size {
                let x = (self.x + j) as usize;
                let y = (self.y + i) as usize;
                if x >= WIDTH as usize || y >= HEIGHT as usize{
                    continue;
                }
                let start: usize = y.wrapping_mul(WIDTH as usize).wrapping_add(x).wrapping_mul(4);
                for count in 0 .. 4 {
                    let index = start + count;
                    if index >= frame.len() {
                        break;
                    }
                    frame[index] = self.color[count];
                }
            }
        }
    }
}

impl MovableMode for Boid {
    fn update(&mut self) {
        if self.x < -SIZE {
            self.x = WIDTH as i16;
        }
        if self.x > WIDTH as i16 {
            self.x = 0;
        }
        if self.y < -SIZE {
            self.y = HEIGHT as i16;
        }
        if self.y > HEIGHT as i16 {
            self.y = 0;
        }
        // if self.x < 0 || self.x + self.size > WIDTH as i16 {
        //     self.velocity_x *= -1;
        // }
        // if self.y < 0 || self.y + self.size > HEIGHT as i16 {
        //     self.velocity_y *= -1;
        // }

        self.x += self.velocity_x;
        self.y += self.velocity_y;
    }
}

struct Background {
    color: [u8; 4],
}

impl Background {
    fn new(color: [u8; 4]) -> Self {
        Self {
            color,
        }
    }
}

impl RenderNode for Background {
    fn draw(&self, frame: &mut[u8]) {
       for pixel in frame.chunks_exact_mut(4) {
           pixel.copy_from_slice(&self.color);
       } 
    }
}
