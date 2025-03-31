use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::KeyEvent,
    ui::Screen,
    widgets::{ConfirmResult, Value, WidgetMut},
};

pub struct Checkbox {
    value: bool,
    style: Style,
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            value: false,
            style: Style::default(),
        }
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn set_value(&mut self, value: bool) {
        self.value = value;
    }

    pub fn toggle(&mut self) {
        self.value = !self.value;
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}

impl WidgetMut<ConfirmResult> for Checkbox {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _: &Screen) {
        Line::raw(if self.value { "[x]" } else { "[ ]" })
            .style(self.style)
            .render(area, buf);
    }

    fn key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        match key {
            _ if key.is_exec() => {
                self.toggle();
                ConfirmResult::Ok
            }
            _ if key.is_back() => ConfirmResult::Cancel,
            _ => ConfirmResult::None,
        }
    }
}

impl Value<bool> for Checkbox {
    fn value(&self) -> bool {
        Self::value(self)
    }

    fn set_value(&mut self, value: bool) {
        Self::set_value(self, value);
    }
}
