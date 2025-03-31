use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_shared::color::RGBA;
use xash3d_ui::raw::HIMAGE;

use crate::{
    input::{Key, KeyEvent},
    ui::Screen,
    widgets::{ConfirmResult, WidgetMut},
};

pub struct Image<'a> {
    picture: HIMAGE,
    colors: &'a [RGBA],
}

impl<'a> Image<'a> {
    pub fn with_color(picture: HIMAGE, colors: &'a [RGBA]) -> Self {
        Self { picture, colors }
    }

    pub fn new(picture: HIMAGE) -> Self {
        Self::with_color(picture, &[])
    }
}

impl WidgetMut<ConfirmResult> for Image<'_> {
    fn render(&mut self, area: Rect, _buf: &mut Buffer, screen: &Screen) {
        if self.picture == 0 {
            return;
        }
        screen.draw_picture(area, self.picture, self.colors);
    }

    fn key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        match key {
            Key::Char(b'q') | Key::Escape => ConfirmResult::Cancel,
            _ => ConfirmResult::None,
        }
    }
}
