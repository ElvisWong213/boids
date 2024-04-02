use rand::Rng;
use node::{ RenderNode, MovableMode };
use crate::node;

#[derive(Clone, PartialEq)]
pub(crate) struct Boid {
    x: i16,
    y: i16,
    size: i16,
    velocity_x: i16,
    velocity_y: i16,
    color: [u8; 4],
    group: u8,
}

impl Boid {
    pub(crate) fn new(x: i16, y: i16, size: i16, velocity_x: i16, velocity_y: i16, color: [u8; 4], group: u8) -> Self {
        Self {
            x,
            y,
            size,
            velocity_x,
            velocity_y,
            color,
            group,
        }
    }

    pub(crate) fn separate(&mut self, boids: &Vec<Boid>, avoid_factor: f32, safe_radius: f32) {
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
            if d <= safe_radius && Boid::in_range(boid_radius, other_boid_radius) {
                close_dx += dx;
                close_dy += dy;
            }
        }
        self.velocity_x += (close_dx * avoid_factor) as i16;
        self.velocity_y += (close_dy * avoid_factor) as i16;
    }

    pub(crate) fn align(&mut self, boids: &Vec<Boid>, matching_factor: f32, vision_radius: f32) {
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
            if d <= vision_radius && Boid::in_range(boid_radius, other_boid_radius) {
                vx_avg += other_boid.velocity_x as f32;
                vy_avg += other_boid.velocity_y as f32;
                neighboring_boids += 1;
            }
        }
        if neighboring_boids > 0 {
            vx_avg /= neighboring_boids as f32;
            vy_avg /= neighboring_boids as f32;
            self.velocity_x += (vx_avg * matching_factor) as i16;
            self.velocity_y += (vy_avg * matching_factor) as i16;
        }
    }

    pub(crate) fn cohesion(&mut self, boids: &Vec<Boid>, centering_factor: f32, vision_radius: f32) {
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
            if d <= vision_radius && Boid::in_range(boid_radius, other_boid_radius) {
                x_avg += other_boid.x as f32;
                y_avg += other_boid.y as f32;
                neighboring_boids += 1;
            }
        }
        if neighboring_boids > 0 {
            x_avg /= neighboring_boids as f32;
            y_avg /= neighboring_boids as f32;
            self.velocity_x += ((x_avg - self.x as f32) * centering_factor) as i16;
            self.velocity_y += ((y_avg - self.y as f32) * centering_factor) as i16;
        }

    }

    pub(crate) fn avoid_border(&mut self, turn_factor: i16, margin: i16, width: u32, height: u32) {
        if self.x < margin {
            self.velocity_x += turn_factor;
        }
        if self.x > width as i16 - margin {
            self.velocity_x -= turn_factor;
        }
        if self.y < margin {
            self.velocity_y += turn_factor;
        }
        if self.y > height as i16 - margin {
            self.velocity_y -= turn_factor;
        }
    }

    pub(crate) fn radius(vx: i16, vy: i16) -> f32 {
        let rad = (vy as f32 / vx as f32).atan();
        if vx >= 0 && vy >= 0 {
            return rad;
        }
        if (vx < 0 && vy >= 0) || (vx < 0 && vy < 0) {
            return 1.0 + rad;
        }
        2.0 + rad
    }

    pub(crate) fn in_range(base: f32, target: f32) -> bool {
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

    pub(crate) fn speed_limit(&mut self, max_speed: i16, min_speed: i16) {
        let x = self.velocity_x.wrapping_mul(self.velocity_x);
        let y = self.velocity_y.wrapping_mul(self.velocity_y);
        let speed = ((x.wrapping_add(y)) as f32).sqrt();
        if speed == 0.0 {
            let mut rng = rand::thread_rng();
            let velocity_x = rng.gen_range(-min_speed..=min_speed);
            let range: [i16; 2] = [-1, 1];
            let velocity_y = ((min_speed.pow(2) - velocity_x.pow(2)) as f32).sqrt() as i16 * range[rng.gen_range(0..=1)];

            self.velocity_x = velocity_x;
            self.velocity_y = velocity_y;
            return;
        }
        if (speed as i16) > max_speed {
            self.velocity_x = ((self.velocity_x as f32 / speed) * max_speed as f32) as i16;
            self.velocity_y = ((self.velocity_y as f32 / speed) * max_speed as f32) as i16;
        }
        if (speed as i16) < min_speed {
            self.velocity_x = ((self.velocity_x as f32 / speed) * min_speed as f32) as i16;
            self.velocity_y = ((self.velocity_y as f32 / speed) * min_speed as f32) as i16;
        }
    }

    pub(crate) fn bias(&mut self, bias_factor: f32) {
        if self.group == 0 {
            self.velocity_y = ((1.0 - bias_factor) * self.velocity_y as f32 + (bias_factor * 1.0)) as i16;
        }
        if self.group == 1 {
            self.velocity_y = ((1.0 - bias_factor) * self.velocity_y as f32 + (bias_factor * -1.0)) as i16;
        }
    }
}

impl RenderNode for Boid {
    fn draw(&self, frame: &mut[u8], width: u32, height: u32) {
        for i in 0..self.size {
            for j in 0..self.size {
                let x = (self.x + j) as usize;
                let y = (self.y + i) as usize;
                if x >= width as usize || y >= height as usize{
                    continue;
                }
                let start: usize = y.wrapping_mul(width as usize).wrapping_add(x).wrapping_mul(4);
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
    fn update(&mut self, width: u32, height: u32) {
        if self.x < -self.size {
            self.x = width as i16;
        }
        if self.x > width as i16 {
            self.x = 0;
        }
        if self.y < -self.size {
            self.y = height as i16;
        }
        if self.y > height as i16 {
            self.y = 0;
        }
        self.x += self.velocity_x;
        self.y += self.velocity_y;
    }
}
