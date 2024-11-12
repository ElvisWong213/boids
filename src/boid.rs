use rand::Rng;
use node::{ RenderNode, MovableMode };
use crate::node::{self, Vertice};

#[derive(Clone, PartialEq)]
pub(crate) struct Boid {
    pub vertice: Vertice,
    size: i16,
    velocity_x: i16,
    velocity_y: i16,
    color: [u8; 4],
}

impl Boid {
    pub(crate) fn new(vertice: Vertice, size: i16, velocity_x: i16, velocity_y: i16, color: [u8; 4]) -> Self {
        Self {
            vertice,
            size,
            velocity_x,
            velocity_y,
            color,
        }
    }

    pub(crate) fn separate(&mut self, boids: &Vec<Boid>, avoid_factor: f32, safe_radius: f32, view_angle: f32) {
        let mut close_dx: f32 = 0.0;
        let mut close_dy: f32 = 0.0;

        let mut new_vertice = Vertice::new();
        new_vertice.x = self.velocity_x + self.vertice.x;
        new_vertice.y = self.velocity_y + self.vertice.y;
        let facing_angle: f32 = Self::angle(&self.vertice, &new_vertice);

        for other_boid in boids {
            if self == other_boid {
                continue;
            }

            let dx = (self.vertice.x - other_boid.vertice.x) as f32;
            let dy = (self.vertice.y - other_boid.vertice.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let object_angle = Self::angle(&self.vertice, &other_boid.vertice);
            if d <= safe_radius && Self::is_within_sight(facing_angle, view_angle, object_angle) {
                close_dx += dx;
                close_dy += dy;
            }
        }
        self.velocity_x += (close_dx * avoid_factor) as i16;
        self.velocity_y += (close_dy * avoid_factor) as i16;
    }

    pub(crate) fn align(&mut self, boids: &Vec<Boid>, matching_factor: f32, vision_radius: f32, view_angle: f32) {
        let mut neighboring_boids: u16 = 0;
        let mut vx_avg: f32 = 0.0;
        let mut vy_avg: f32 = 0.0;

        let mut new_vertice = Vertice::new();
        new_vertice.x = self.velocity_x + self.vertice.x;
        new_vertice.y = self.velocity_y + self.vertice.y;
        let facing_angle: f32 = Self::angle(&self.vertice, &new_vertice);

        for other_boid in boids {
            if self == other_boid {
                continue;
            }
            let dx = (self.vertice.x - other_boid.vertice.x) as f32;
            let dy = (self.vertice.y - other_boid.vertice.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let object_angle = Self::angle(&self.vertice, &other_boid.vertice);
            if d <= vision_radius && Self::is_within_sight(facing_angle, view_angle, object_angle) {
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

    pub(crate) fn cohesion(&mut self, boids: &Vec<Boid>, centering_factor: f32, vision_radius: f32, view_angle: f32) {
        let mut neighboring_boids: u16 = 0;
        let mut x_avg: f32 = 0.0;
        let mut y_avg: f32 = 0.0;

        let mut new_vertice = Vertice::new();
        new_vertice.x = self.velocity_x + self.vertice.x;
        new_vertice.y = self.velocity_y + self.vertice.y;
        let facing_angle: f32 = Self::angle(&self.vertice, &new_vertice);

        for other_boid in boids {
            if self == other_boid {
                continue;
            }
            let dx = (self.vertice.x - other_boid.vertice.x) as f32;
            let dy = (self.vertice.y - other_boid.vertice.y) as f32;
            let d = (dx * dx + dy * dy).sqrt();
            let object_angle = Self::angle(&self.vertice, &other_boid.vertice);
            if d <= vision_radius && Self::is_within_sight(facing_angle, view_angle, object_angle) {
                x_avg += other_boid.vertice.x as f32;
                y_avg += other_boid.vertice.y as f32;
                neighboring_boids += 1;
            }
        }
        if neighboring_boids > 0 {
            x_avg /= neighboring_boids as f32;
            y_avg /= neighboring_boids as f32;
            self.velocity_x += ((x_avg - self.vertice.x as f32) * centering_factor) as i16;
            self.velocity_y += ((y_avg - self.vertice.y as f32) * centering_factor) as i16;
        }

    }

    pub(crate) fn avoid_border(&mut self, turn_factor: i16, margin: i16, width: u16, height: u16) {
        if self.vertice.x < margin {
            self.velocity_x += turn_factor;
        }
        if self.vertice.x > width as i16 - margin {
            self.velocity_x -= turn_factor;
        }
        if self.vertice.y < margin {
            self.velocity_y += turn_factor;
        }
        if self.vertice.y > height as i16 - margin {
            self.velocity_y -= turn_factor;
        }
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

    pub(crate) fn noise(&mut self, on: bool) {
        if !on {
            return;
        }
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(0.0..1.0);
        let x_val = if rng.gen_bool(0.5) {
            1.0 * val * self.velocity_x as f32
        } else {
            -1.0 * val * self.velocity_x as f32
        };
        let y_val = if rng.gen_bool(0.5) {
            1.0 * val * self.velocity_y as f32
        } else {
            -1.0 * val * self.velocity_y as f32
        };
        self.velocity_x += x_val as i16;
        self.velocity_y += y_val as i16;
    }

    pub(crate) fn update_color(&mut self, max_speed: i16, min_speed: i16) {
        let velocity_x = self.velocity_x as f32;
        let velocity_y = self.velocity_y as f32;
        let mut current_speed = velocity_x * velocity_x + velocity_y * velocity_y;
        current_speed = current_speed.sqrt();
        let max = max_speed as f32;
        let min = min_speed as f32;
        if current_speed > max {
            return;
        }
        let range = (current_speed - min) / (max - min) * 255.0;
        self.color[0] = 255 - range as u8;
        self.color[1] = range as u8;
    }

    // right is 0 degree
    fn angle(origin: &Vertice, other: &Vertice) -> f32 {
        let dx = (origin.x - other.x) as f32;
        let dy = (origin.y - other.y) as f32;
        let radions = (dy / dx).atan() * 180.0;
        if dx >= 0.0 && dy >= 0.0 {
            return radions;
        }
        if dx < 0.0 && dy >= 0.0 {
            return 180.0 - radions;
        }
        if dx < 0.0 && dy < 0.0 {
            return 180.0 + radions;
        }
        360.0 - radions
    }

    fn is_within_sight(facing_angle: f32, view_angle: f32, object_angle: f32) -> bool {
        let lower_angle: f32 = facing_angle - (view_angle / 2.0);
        let upper_angle: f32 = facing_angle + (view_angle / 2.0);
        let lower_to_object: f32 = (object_angle - lower_angle) % 360.0;
        let lower_to_upper: f32 = (upper_angle - lower_angle) % 360.0;
        if lower_to_object <= lower_to_upper {
            return true;
        }
        false
    }

}

impl RenderNode for Boid {
    fn draw(&self, frame: &mut[u8], width: u16, height: u16) {
        for i in 0..self.size {
            for j in 0..self.size {
                let x = (self.vertice.x + j) as usize;
                let y = (self.vertice.y + i) as usize;
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
    fn update(&mut self, width: u16, height: u16) {
        self.vertice.x += self.velocity_x;
        self.vertice.y += self.velocity_y;
        if self.vertice.x < -self.size {
            self.vertice.x = width as i16;
        }
        if self.vertice.x > width as i16 {
            self.vertice.x = 0;
        }
        if self.vertice.y < -self.size {
            self.vertice.y = height as i16;
        }
        if self.vertice.y > height as i16 {
            self.vertice.y = 0;
        }
    }
}
