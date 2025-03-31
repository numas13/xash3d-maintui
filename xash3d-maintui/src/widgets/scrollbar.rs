use ratatui::{
    prelude::*,
    widgets::{ScrollbarOrientation, ScrollbarState},
};

pub struct Scrollbar {
    style: Style,
    thumb_style: Style,
    offset: usize,
    len: usize,
    extra: usize,
}

impl Scrollbar {
    pub fn new(offset: usize, len: usize, extra: usize) -> Self {
        Self {
            style: Style::default().gray(),
            thumb_style: Style::default().yellow(),
            offset,
            len,
            extra,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn thumb_style(mut self, style: Style) -> Self {
        self.thumb_style = style;
        self
    }
}

impl Widget for Scrollbar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let scrollbar = ratatui::widgets::Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(self.style)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("\u{2502}"))
            .thumb_symbol("\u{2551}")
            .thumb_style(self.thumb_style);

        let height = (area.height as usize).saturating_sub(self.extra);
        let view_height = self.len.saturating_sub(height);
        let mut scrollbar_state = ScrollbarState::new(view_height).position(self.offset);

        // let area = area.inner(Margin {
        //     vertical: 1,
        //     horizontal: 0,
        // });
        scrollbar.render(area, buf, &mut scrollbar_state);
    }
}
