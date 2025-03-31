use std::ffi::c_int;

use ratatui::layout::{Position, Rect, Size};
use xash3d_ratatui::XashBackend;
use xash3d_shared::{color::RGBA, raw::wrect_s};
use xash3d_ui::engine;

pub struct Screen {
    /// Cell size in pixels.
    pub cell: Size,
    /// Alignment offset in pixels.
    pub align: Position,
}

impl Screen {
    pub fn new(backend: &XashBackend) -> Self {
        Screen {
            cell: backend.cell_size_in_pixels(),
            align: backend.alignment_offset(),
        }
    }

    pub fn draw_picture(&self, area: Rect, pic: c_int, colors: &[RGBA]) {
        let engine = engine();
        let mut x = (self.align.x + self.cell.width * area.x) as c_int;
        let mut y = (self.align.y + self.cell.height * area.y) as c_int;
        let mut w = (area.width * self.cell.width) as c_int;
        let mut h = (area.height * self.cell.height) as c_int;
        engine.fill_rgba((x, y), (w, h), RGBA::BLACK);

        let size = engine.pic_size(pic);
        let r = size.width as f32 / size.height as f32;
        if (w as f32 / h as f32) < r {
            let t = (w as f32 / r) as c_int;
            y += (h - t) / 2;
            h = t;
        } else {
            let t = (h as f32 * r) as c_int;
            x += (w - t) / 2;
            w = t;
        }
        if colors.is_empty() {
            engine.pic_set(pic, RGBA::WHITE);
            engine.pic_draw((x, y), (w, h), None);
        } else {
            let len = colors.len() as f64;
            let y_step = h as f64 / len;
            let r_step = size.height as f64 / len;
            for (i, color) in colors.iter().enumerate() {
                let i = i as f64;
                let rect = wrect_s {
                    left: 0,
                    right: size.width,
                    top: (i * r_step).round() as c_int,
                    bottom: ((i + 1.0) * r_step).round() as c_int,
                };
                let y = y + (i * y_step).round() as c_int;
                engine.pic_set(pic, *color);
                engine.pic_draw((x, y), (w, y_step as c_int), Some(&rect));
            }
        }
    }
}
