use crate::boid::Boid;
use std::fmt::Display;

pub(crate) trait RenderNode {
    fn draw(&self, _frame: &mut [u8], _width: u16, _height: u16) {}
}

pub(crate) trait MovableNode {
    fn update(&mut self, _width: u16, _height: u16) {}
}

#[derive(Clone, PartialEq)]
pub(crate) struct Vertice {
    pub x: i16,
    pub y: i16,
}

impl Vertice {
    pub(crate) fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    pub(crate) fn slope(&self, other: &Vertice) -> Option<f32> {
        let y_diff = self.y - other.y;
        let x_diff = self.x - other.x;
        if x_diff == 0 {
            return None;
        }
        Some(y_diff as f32 / x_diff as f32)
    }
}

impl Display for Vertice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.x, self.y)
    }
}

#[derive(Clone)]
pub(crate) struct Rectangle {
    center_x: f32,
    center_y: f32,
    half_width: f32,
    half_height: f32,
}

impl Rectangle {
    pub(crate) fn new(center_x: f32, center_y: f32, half_width: f32, half_height: f32) -> Self {
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

impl RenderNode for Rectangle {
    fn draw(&self, _frame: &mut [u8], _width: u16, _height: u16) {
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

        draw_line(&a, &b, _frame, _width, _height);
        draw_line(&b, &c, _frame, _width, _height);
        draw_line(&c, &d, _frame, _width, _height);
        draw_line(&d, &a, _frame, _width, _height);
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

#[derive(Clone)]
pub(crate) struct QuadTree {
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
    pub(crate) fn new(capacity: usize, boundary: Rectangle) -> Self {
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

    pub(crate) fn insert(&mut self, boid: &Boid) -> bool {
        if !self
            .boundary
            .contains_point(boid.vertice.x as f32, boid.vertice.y as f32)
        {
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
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &mut self.top_right {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &mut self.bottom_left {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &mut self.bottom_right {
            Some(q_tree) => {
                if q_tree.insert(boid) {
                    return true;
                }
            }
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

    pub(crate) fn query(&self, found: &mut Vec<Boid>, boid: &Boid, vision_radius: f32) {
        if !self.boundary.intersects(&boid.vertice, vision_radius)
            && !self
                .boundary
                .contains_point(boid.vertice.x as f32, boid.vertice.y as f32)
        {
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
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.query(found, boid, vision_radius);
            }
            None => {
                panic!("Bottom right is not create");
            }
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<Boid> {
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
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                boids.append(&mut q_tree.to_vec());
            }
            None => {
                panic!("Bottom right is not create");
            }
        }
        boids
    }

    pub(crate) fn draw_quad_tree(&self, _frame: &mut [u8], _width: u16, _height: u16) {
        self.boundary.draw(_frame, _width, _height);
        if !self.splitted {
            return;
        }
        match &self.top_left {
            Some(q_tree) => {
                q_tree.draw_quad_tree(_frame, _width, _width);
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.draw_quad_tree(_frame, _width, _width);
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.draw_quad_tree(_frame, _width, _width);
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.draw_quad_tree(_frame, _width, _width);
            }
            None => {
                panic!("Bottom right is not create");
            }
        }
    }
}

impl RenderNode for QuadTree {
    fn draw(&self, _frame: &mut [u8], _width: u16, _height: u16) {
        for boid in &self.boids {
            boid.draw(_frame, _width, _width);
        }
        if !self.splitted {
            return;
        }
        match &self.top_left {
            Some(q_tree) => {
                q_tree.draw(_frame, _width, _width);
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.draw(_frame, _width, _width);
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.draw(_frame, _width, _width);
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.draw(_frame, _width, _width);
            }
            None => {
                panic!("Bottom right is not create");
            }
        }
    }
}

pub(crate) fn draw_line(start: &Vertice, end: &Vertice, frame: &mut [u8], width: u16, height: u16) {
    let color  = Color::White.to_color_array();
    match start.slope(end) {
        Some(slope) => {
            if slope == 0.0 {
                for x in start.x..=end.x {
                    change_pixel(frame, x as usize, start.y as usize, width, height, color);
                }
            } else {
                for x in start.x..=end.x {
                    let y = (slope * x as f32) as usize;
                    change_pixel(frame, x as usize, y, width, height, color);
                }
            }
        }
        None => {
            for y in start.y..=end.y {
                change_pixel(frame, start.x as usize, y as usize, width, height, color);
            }
        }
    };
}

pub(crate) fn change_pixel(frame: &mut [u8], x: usize, y: usize, width: u16, height: u16, color: [u8; 4]) {
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

enum Color {
    White,
    Red,
    Green,
    Blue,
}

impl Color {
    pub(crate) fn to_color_array(&self) -> [u8; 4] {
        match self {
            Color::White => {
                [255, 255, 255, 255]
            }
            Color::Red => {
                [255, 0, 0, 255]
            }
            Color::Green => {
                [0, 255, 0, 255]
            }
            Color::Blue => {
                [0, 0, 255, 255]
            }        
        }
    }
}
