use ratatui::prelude::*;

use crate::{i18n, strings::strings};

pub struct Button {
    pub area: Rect,
    pub label: String,
}

impl Button {
    pub fn new(label: &str) -> Self {
        Self {
            area: Rect::ZERO,
            label: format!("{label:^21}"),
        }
    }

    pub fn cancel() -> Self {
        Self::new(strings().get(i18n::all::CANCEL))
    }

    pub fn yes() -> Self {
        Self::new(strings().get(i18n::all::YES))
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, focused: bool) {
        let style = if focused {
            Style::default().black().on_yellow()
        } else {
            Style::default().white().on_black()
        };
        Line::raw(&self.label)
            .style(style)
            .centered()
            .render(area, buf);
        self.area = area;
    }
}
