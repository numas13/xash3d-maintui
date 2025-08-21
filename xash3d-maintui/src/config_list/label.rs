use compact_str::{CompactString, ToCompactString};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{input::KeyEvent, ui::Screen};

use super::{ConfigAction, ConfigItem};

pub struct Label {
    label: CompactString,
}

impl Label {
    pub fn new(label: impl ToCompactString) -> Self {
        Self {
            label: label.to_compact_string(),
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
