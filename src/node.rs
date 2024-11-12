use std::fmt::Display;

pub(crate) trait RenderNode {
    fn draw(&self, _frame: &mut[u8], _width: u16, _height: u16) {}
}

pub(crate) trait MovableMode {
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
}

impl Display for Vertice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.x, self.y)
    }
}
