use crate::node::RenderNode;

#[derive(Clone)]
pub(crate) struct Background {
    color: [u8; 4],
}

impl Background {
    pub(crate) fn new(color: [u8; 4]) -> Self {
        Self {
            color,
        }
    }
}

impl RenderNode for Background {
    fn draw(&self, frame: &mut[u8], _width: u32, _height: u32) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&self.color);
        }
    }
}
