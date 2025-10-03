use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    prelude::*,
    strings::Localize,
    ui::{Control, Menu, Screen},
};

mod i18n {
    pub use crate::i18n::menu::config_game::*;
}

pub struct GameConfig {
    list: ConfigList,
}

impl GameConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());

        let engine = engine();
        let info = engine.game_info2().unwrap();
        if info.game_dir() == c"cstrike" {
            list.slider(i18n::WEAPON_LAG.localize(), c"cl_weaponlag");
        }

        Self { list }
    }
}

impl Menu for GameConfig {
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
