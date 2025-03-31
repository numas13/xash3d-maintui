use ratatui::prelude::*;

pub struct Button {
    pub area: Rect,
    pub label: &'static str,
}

impl Button {
    pub fn new(label: &'static str) -> Self {
        Self {
            area: Rect::ZERO,
            label,
        }
    }

    pub fn cancel() -> Self {
        Self::new("        Cancel        ")
    }

    pub fn yes() -> Self {
        Self::new("         Yes         ")
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, focused: bool) {
        let style = if focused {
            Style::default().black().on_yellow()
        } else {
            Style::default().white().on_black()
        };
        Line::raw(self.label)
            .style(style)
            .centered()
            .render(area, buf);
        self.area = area;
    }
}
