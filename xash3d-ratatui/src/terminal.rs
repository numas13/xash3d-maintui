use ratatui::{buffer::Buffer, layout::Rect};

use crate::XashBackend;

#[derive(Default)]
pub struct XashTerminal {
    backend: XashBackend,
    buffer: Buffer,
}

impl XashTerminal {
    pub fn backend(&self) -> &XashBackend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut XashBackend {
        &mut self.backend
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.backend.resize(width, height);
        self.buffer.resize(self.backend.area());
    }

    pub fn draw<F>(&mut self, mut render_callback: F)
    where
        F: FnMut(Rect, &mut Buffer, &mut XashBackend),
    {
        self.buffer.reset();
        let area = self.backend.area();
        render_callback(area, &mut self.buffer, &mut self.backend);
        self.backend.draw_buffer(&self.buffer);
    }
}
