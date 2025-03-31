use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    strings::strings,
    ui::{Control, Menu, Screen},
};

pub struct VoiceConfig {
    list: ConfigList,
}

impl VoiceConfig {
    pub fn new() -> Self {
        let strings = strings();
        let l = |s| strings.get(s);
        let mut list = ConfigList::with_back("Voice settings");

        list.checkbox(l("#GameUI_EnableVoice"), c"voice_modenable");
        list.slider(l("#GameUI_VoiceTransmitVolume"), c"voice_transmit_scale");
        list.slider(l("#GameUI_VoiceReceiveVolume"), c"voice_scale");
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
