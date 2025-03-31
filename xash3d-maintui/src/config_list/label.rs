use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{input::KeyEvent, ui::Screen};

use super::{ConfigAction, ConfigItem};

pub struct Label {
    label: String,
}

impl Label {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
        }
    }
}

impl ConfigItem for Label {
    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, _: &Screen, style: Style) {
        Line::raw(self.label.as_str())
            .style(style)
            .render(area, buf);
    }

    fn item_key_event(&mut self, _: &XashBackend, _: KeyEvent) -> ConfigAction {
        ConfigAction::None
    }
}
