use crate::{geometry::Color, node::RenderNode};

pub struct Background {
    color: Color,
}

impl Background {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl RenderNode for Background {
    fn draw(&self, _frame: &mut [u8], _width: u16, _height: u16) {
        let color = self.color.to_color_array();
        for pixel in _frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
    }
}
