use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    ui::{Control, Menu, Screen},
};

pub struct AudioConfig {
    list: ConfigList,
}

impl AudioConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back("Audio settings");
        list.slider("Sound effects volume", c"volume");
        list.slider("MP3 Volume", c"MP3Volume");
        list.slider("HEV suit volume", c"suitvolume");
        list.popup_list(
            "Sound interpolation",
            c"s_lerping",
            ["Disable", "Balance", "Quality"],
        );
        list.checkbox("Mute when inactive", c"snd_mute_losefocus");
        list.checkbox("Disable DSP effects", c"room_off");
        list.checkbox("Use Alpha DSP effects", c"dsp_coeff_table");
        list.checkbox("Enable vibration", c"vibration_enable");
        list.slider("Vibration", c"vibration_length");

        Self { list }
    }
}

impl Menu for AudioConfig {
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
