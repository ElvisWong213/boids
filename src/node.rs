pub(crate) trait RenderNode {
    fn draw(&self, _frame: &mut[u8], _width: u32, _height: u32) {}
}

pub(crate) trait MovableMode {
    fn update(&mut self, _width: u32, _height: u32) {}
}
