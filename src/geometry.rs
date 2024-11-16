use std::{f32::consts::PI, fmt::Display, mem::swap};

use crate::node::{RenderNode, Vertice};

#[derive(Clone)]
pub struct Rectangle {
    pub center_x: f32,
    pub center_y: f32,
    pub half_width: f32,
    pub half_height: f32,
}

impl Rectangle {
    pub fn new(center_x: f32, center_y: f32, half_width: f32, half_height: f32) -> Self {
        Self {
            center_x,
            center_y,
            half_width,
            half_height,
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let min_x = self.center_x - self.half_width;
        let max_x = self.center_x + self.half_width;
        let min_y = self.center_y - self.half_height;
        let max_y = self.center_y + self.half_height;
        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    pub fn intersects(&self, vertice: &Vertice, vision_radius: f32) -> bool {
        let dx = self.center_x - vertice.x as f32;
        let dy = self.center_y - vertice.y as f32;
        let d = (dx * dx + dy * dy).sqrt();
        d <= vision_radius
    }
}

impl RenderNode for Rectangle {
    fn draw(&self, frame: &mut [u8], width: u16, height: u16) {
        let mut a = Vertice::new();
        let mut b = Vertice::new();
        let mut c = Vertice::new();
        let mut d = Vertice::new();

        a.x = self.center_x as i16 - self.half_width as i16;
        a.y = self.center_y as i16 - self.half_height as i16;

        b.x = self.center_x as i16 - self.half_width as i16;
        b.y = self.center_y as i16 + self.half_height as i16;

        c.x = self.center_x as i16 + self.half_width as i16;
        c.y = self.center_y as i16 + self.half_height as i16;

        d.x = self.center_x as i16 + self.half_width as i16;
        d.y = self.center_y as i16 - self.half_height as i16;

        draw_line(&a, &b, frame, width, height);
        draw_line(&b, &c, frame, width, height);
        draw_line(&c, &d, frame, width, height);
        draw_line(&d, &a, frame, width, height);
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "x: {}, y: {}, width: {}, height: {}",
            self.center_x, self.center_y, self.half_width, self.half_height
        )
    }
}

pub fn draw_line(start: &Vertice, end: &Vertice, frame: &mut [u8], width: u16, height: u16) {
    let color = Color::White.to_color_array();
    let mut start_x = start.x;
    let mut start_y = start.y;
    let mut end_x = end.x;
    let mut end_y = end.y;
    sort_two_value(&mut start_x, &mut end_x);
    sort_two_value(&mut start_y, &mut end_y);
    match start.slope(end) {
        Some(slope) => {
            if slope == 0.0 {
                for x in start_x..=end_x {
                    change_pixel(frame, x as usize, start.y as usize, width, height, color);
                }
            } else {
                let c = start.y as f32 - slope * start.x as f32; 
                for x in start_x..=end_x {
                    let y = (slope * x as f32 + c) as usize;
                    change_pixel(frame, x as usize, y, width, height, color);
                }
            }
        }
        None => {
            for y in start_y..=end_y {
                change_pixel(frame, start.x as usize, y as usize, width, height, color);
            }
        }
    };
}

pub fn sort_two_value(val_a: &mut i16, val_b: &mut i16) {
    if val_a > val_b {
        swap(val_a, val_b);
    }
}

pub fn change_pixel(
    frame: &mut [u8],
    x: usize,
    y: usize,
    width: u16,
    height: u16,
    color: [u8; 4],
) {
    if x > width as usize || y > height as usize {
        return;
    }
    let start: usize = y
        .wrapping_mul(width as usize)
        .wrapping_add(x)
        .wrapping_mul(4);
    for (count, val) in color.iter().enumerate() {
        let index = start + count;
        if index >= frame.len() {
            break;
        }
        frame[index] = *val;
    }
}

pub struct Circle {
    x: f32,
    y: f32,
    radius: f32,
    color: Color,
}

impl Circle {
    pub fn new(x: f32, y: f32, radius: f32, color: Color) -> Self {
        Self { x, y, radius, color }
    }
}

impl RenderNode for Circle {
    fn draw(&self, _frame: &mut [u8], _width: u16, _height: u16) {
        if self.radius == 0.0 {
            return;
        }
        let color = self.color.to_color_array();
        for angle in 0..=365 {
            let angle = angle as f32;
            let x = self.radius * (angle * PI / 180.0).cos() + self.x;
            let y = self.radius * (angle * PI / 180.0).sin() + self.y;
            change_pixel(_frame, x as usize, y as usize, _width, _height, color);
        }
    }
}

#[derive(Clone)]
pub enum Color {
    Black,
    White,
    Red,
    Green,
    Blue,
}

impl Color {
    pub fn to_color_array(&self) -> [u8; 4] {
        match self {
            Color::Black => [0, 0, 0, 0],
            Color::White => [255, 255, 255, 255],
            Color::Red => [255, 0, 0, 255],
            Color::Green => [0, 255, 0, 255],
            Color::Blue => [0, 0, 255, 255],
        }
    }
}
