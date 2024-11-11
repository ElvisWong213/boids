pub(crate) trait RenderNode {
    fn draw(&self, _frame: &mut[u8], _width: u32, _height: u32) {}
}

pub(crate) trait MovableMode {
    fn update(&mut self, _width: u32, _height: u32) {}
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
