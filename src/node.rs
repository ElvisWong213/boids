use crate::{boid::Boid, geometry::Rectangle, WorldOption};
use std::fmt::Display;

pub(crate) trait RenderNode {
    fn draw_with_option(&self, _frame: &mut [u8], _width: u16, _height: u16, _world_option: &WorldOption) {}
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

    pub(crate) fn draw_quad_tree(&self, frame: &mut [u8], width: u16, height: u16) {
        self.boundary.draw(frame, width, height);
    }
}

impl RenderNode for QuadTree {
    fn draw_with_option(&self, frame: &mut [u8], width: u16, height: u16, _world_option: &WorldOption) {
        if _world_option.show_quad_tree {
            self.draw_quad_tree(frame, width, height);
        }
        for boid in &self.boids {
            boid.draw_with_option(frame, width, width, _world_option);
        }
        if !self.splitted {
            return;
        }
        match &self.top_left {
            Some(q_tree) => {
                q_tree.draw_with_option(frame, width, width, _world_option);
            }
            None => {
                panic!("Top left is not create");
            }
        }
        match &self.top_right {
            Some(q_tree) => {
                q_tree.draw_with_option(frame, width, width, _world_option);
            }
            None => {
                panic!("Top right is not create");
            }
        }
        match &self.bottom_left {
            Some(q_tree) => {
                q_tree.draw_with_option(frame, width, width, _world_option);
            }
            None => {
                panic!("Bottom left is not create");
            }
        }
        match &self.bottom_right {
            Some(q_tree) => {
                q_tree.draw_with_option(frame, width, width, _world_option);
            }
            None => {
                panic!("Bottom right is not create");
            }
        }
    }
}
