use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{Control, Screen},
};

use super::{ConfigAction, ConfigItem};

pub struct Button<T> {
    label: String,
    func: T,
}

impl<T> Button<T> {
    pub fn new(label: impl ToString, func: T) -> Self {
        Self {
            label: label.to_string(),
            func,
        }
    }
}

impl<T> ConfigItem for Button<T>
where
    T: FnMut() -> Control,
{
    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, _: &Screen, style: Style) {
        Line::raw(&self.label).style(style).render(area, buf);
    }

    fn item_key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfigAction {
        let key = event.key();
        if key.is_exec() || matches!(key, Key::Mouse(0)) {
            ConfigAction::Control((self.func)())
        } else {
            ConfigAction::None
        }
    }
}
