use ratatui::{buffer::Buffer, layout::Rect};
use xash3d_ui::prelude::*;

use crate::XashBackend;

pub struct XashTerminal {
    backend: XashBackend,
    buffer: Buffer,
}

impl XashTerminal {
    pub fn new(engine: UiEngineRef) -> Self {
        Self {
            backend: XashBackend::new(engine),
            buffer: Buffer::default(),
        }
    }

    pub fn backend(&self) -> &XashBackend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut XashBackend {
        &mut self.backend
    }

    pub fn resize(&mut self, width: u32, height: u32) {
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
