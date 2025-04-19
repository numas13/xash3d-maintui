use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    strings::Localize,
    ui::{Control, Menu, Screen},
};

mod i18n {
    pub use crate::i18n::menu::config_audio::*;
}

pub struct AudioConfig {
    list: ConfigList,
}

impl AudioConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());
        list.slider(i18n::SOUND_EFFECTS_VOLUME.localize(), c"volume");
        list.slider(i18n::MP3_VOLUME.localize(), c"MP3Volume");
        list.slider(i18n::HEV_SUIT_VOLUME.localize(), c"suitvolume");
        list.popup_list(
            i18n::SOUND_INTERPOLATION.localize(),
            c"s_lerping",
            [
                i18n::SOUND_INTERPOLATION_DISABLE.localize(),
                i18n::SOUND_INTERPOLATION_BALANCE.localize(),
                i18n::SOUND_INTERPOLATION_QUALITY.localize(),
            ],
        );
        list.checkbox(i18n::MUTE_INACTIVE.localize(), c"snd_mute_losefocus");
        list.checkbox(i18n::DISABLE_DSP_EFFECTS.localize(), c"room_off");
        list.checkbox(i18n::ALPHA_DSP_EFFECTS.localize(), c"dsp_coeff_table");
        list.checkbox(i18n::ENABLE_VIBRATION.localize(), c"vibration_enable");
        list.slider(i18n::VIBRATION.localize(), c"vibration_length");
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
