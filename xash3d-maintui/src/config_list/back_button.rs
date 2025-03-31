use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{Control, Screen},
};

use super::{ConfigAction, ConfigItem};

pub struct BackButton;

impl ConfigItem for BackButton {
    fn get_hint(&self) -> Option<&str> {
        Some("Return to previous menu.")
    }

    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, _: &Screen, style: Style) {
        Line::raw("Back").style(style).render(area, buf);
    }

    fn item_key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfigAction {
        let key = event.key();
        if key.is_exec() || matches!(key, Key::Mouse(0)) {
            ConfigAction::Control(Control::Back)
        } else {
            ConfigAction::None
        }
    }
}
