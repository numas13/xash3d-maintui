use core::ffi::c_int;

use ratatui::layout::{Position, Rect, Size};
use xash3d_ratatui::XashBackend;
use xash3d_ui::{color::RGBA, misc::Rect as UiRect, picture::Picture};

use crate::prelude::*;

pub struct Screen {
    /// Cell size in pixels.
    pub cell: Size,
    pub cursor: Position,
}

impl Screen {
    pub fn new(backend: &XashBackend) -> Self {
        Screen {
            cell: backend.cell_size_in_pixels(),
            cursor: backend.cursor_position(),
        }
    }

    pub fn area_to_pixels(&self, area: Rect) -> Rect {
        Rect {
            x: area.x * self.cell.width,
            y: area.y * self.cell.height,
            width: area.width * self.cell.width,
            height: area.height * self.cell.height,
        }
    }

    pub fn draw_picture(&self, area: Rect, pic: Picture, colors: &[RGBA]) {
        let engine = engine();
        let area = self.area_to_pixels(area);
        let mut x = area.x as i32;
        let mut y = area.y as i32;
        let mut w = area.width as u32;
        let mut h = area.height as u32;
        engine.fill_rgba(RGBA::BLACK, UiRect::new(x, y, w, h));

        let size = pic.size();
        let r = size.width as f32 / size.height as f32;
        if (w as f32 / h as f32) < r {
            let t = (w as f32 / r) as u32;
            y += ((h - t) / 2) as i32;
            h = t;
        } else {
            let t = (h as f32 * r) as u32;
            x += ((w - t) / 2) as i32;
            w = t;
        }
        if colors.is_empty() {
            let area = UiRect::new(x, y, w, h);
            pic.draw(RGBA::WHITE, area, None);
        } else {
            let len = colors.len() as f64;
            let y_step = h as f64 / len;
            let r_step = size.height as f64 / len;
            for (i, color) in colors.iter().enumerate() {
                let i = i as f64;
                let pic_area = UiRect::new(
                    0,
                    (i * r_step).round() as i32,
                    size.width,
                    ((i + 1.0) * r_step).round() as u32,
                );
                let y = y + (i * y_step).round() as c_int;
                let area = UiRect::new(x, y, w, y_step as u32);
                pic.draw(*color, area, Some(pic_area));
            }
        }
    }
}
