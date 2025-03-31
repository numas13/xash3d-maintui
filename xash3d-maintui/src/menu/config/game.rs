use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::engine;

use crate::{
    config_list::ConfigList,
    input::KeyEvent,
    ui::{Control, Menu, Screen},
};

pub struct GameConfig {
    list: ConfigList,
}

impl GameConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back("Game settings");

        let info = engine().get_game_info_2().unwrap();
        if info.gamefolder.as_c_str() == c"cstrike" {
            list.slider("Weapon lag", c"cl_weaponlag");
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
