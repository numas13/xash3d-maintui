use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{color::RGBA, picture::Picture};

use crate::{
    input::{Key, KeyEvent},
    ui::Screen,
    widgets::{ConfirmResult, WidgetMut},
};

pub struct Image<'a> {
    pic: Picture,
    colors: &'a [RGBA],
}

impl<'a> Image<'a> {
    pub fn with_color(pic: Picture, colors: &'a [RGBA]) -> Self {
        Self { pic, colors }
    }

    pub fn new(pic: Picture) -> Self {
        Self::with_color(pic, &[])
    }
}

impl WidgetMut<ConfirmResult> for Image<'_> {
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        for pos in area.positions() {
            buf[pos].reset();
        }
        if !self.pic.is_none() {
            screen.draw_picture(area, self.pic, self.colors);
        }
    }

    fn key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        match key {
            Key::Char(b'q') | Key::Escape => ConfirmResult::Cancel,
            _ => ConfirmResult::None,
        }
    }
}
