use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    strings::Localize,
    ui::{Control, Menu, Screen},
};

mod i18n {
    pub use crate::i18n::menu::config_voice::*;
}

pub struct VoiceConfig {
    list: ConfigList,
}

impl VoiceConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());
        list.checkbox(i18n::ENABLE_VOICE.localize(), c"voice_modenable");
        list.slider(
            i18n::VOICE_TRANSMIT_VOLUME.localize(),
            c"voice_transmit_scale",
        );
        list.slider(i18n::VOICE_RECEIVE_VOLUME.localize(), c"voice_scale");
        list.label("* Uses Opus Codec.");
        list.label("* Open, royalty-free, highly versatile audio codec.");
        Self { list }
    }
}

impl Menu for VoiceConfig {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.list.draw_centered(area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        self.list.key_event(backend, event)
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.list.mouse_event(backend)
    }
}
