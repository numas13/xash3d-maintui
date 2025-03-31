mod audio;
mod game;
mod gamepad;
mod keyboard;
mod mouse;
mod multiplayer;
mod network;
mod touch_buttons;
mod video;
mod voice;

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    menu::define_menu_items,
    ui::{utils, Control, Menu, Screen},
    widgets::{List, WidgetMut},
};

define_menu_items! {
    MENU_KEYBOARD = "Keyboard", "Change keyboard settings.";
    MENU_GAMEPAD = "Gamepad", "Change gamepad settings.";
    MENU_MOUSE = "Mouse", "Change mouse settings.";
    MENU_GAME = "Game", "Change game settings.";
    MENU_MULTIPLAYER = "Multiplayer", "Change multiplayer settings.";
    MENU_AUDIO = "Audio", "Change audio settings.";
    MENU_VOICE = "Voice", "Change voice settings.";
    MENU_VIDEO = "Video", "Change video settings.";
    MENU_NETWORK = "Network", "Change network settings.";
    MENU_TOUCH_BUTTONS = "Touch buttons", "Change touch buttons.";
    MENU_BACK = "Back", "Go back to the Main menu.";
}

pub struct ConfigMenu {
    menu: List,
}

impl ConfigMenu {
    pub fn new() -> Self {
        let mut menu = List::empty();
        menu.push(MENU_KEYBOARD);
        menu.push(MENU_MOUSE);
        menu.push(MENU_GAMEPAD);
        menu.push(MENU_GAME);
        menu.push(MENU_MULTIPLAYER);
        menu.push(MENU_VOICE);
        menu.push(MENU_AUDIO);
        menu.push(MENU_VIDEO);
        menu.push(MENU_NETWORK);
        if utils::is_dev() {
            menu.push(MENU_TOUCH_BUTTONS);
        }
        menu.push(MENU_BACK);
        menu.set_bindings([
            (Key::Char(b'e'), MENU_KEYBOARD),
            (Key::Char(b'm'), MENU_MOUSE),
            (Key::Char(b'p'), MENU_GAMEPAD),
            (Key::Char(b'g'), MENU_GAME),
            (Key::Char(b'a'), MENU_AUDIO),
            (Key::Char(b'v'), MENU_VIDEO),
            (Key::Char(b'n'), MENU_NETWORK),
            (Key::Char(b'b'), MENU_BACK),
        ]);

        Self { menu }
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_GAME => Control::next(game::GameConfig::new()),
            MENU_MULTIPLAYER => Control::next(multiplayer::MultiplayerConfig::new()),
            MENU_AUDIO => Control::next(audio::AudioConfig::new()),
            MENU_VOICE => Control::next(voice::VoiceConfig::new()),
            MENU_VIDEO => Control::next(video::VideoConfig::new()),
            MENU_KEYBOARD => Control::next(keyboard::Controls::new()),
            MENU_MOUSE => Control::next(mouse::MouseConfig::new()),
            MENU_GAMEPAD => Control::next(gamepad::GamepadConfig::new()),
            MENU_NETWORK => Control::next(network::NetworkConfig::new()),
            MENU_TOUCH_BUTTONS => Control::next(touch_buttons::TouchButtonsConfig::new()),
            MENU_BACK => Control::Back,
            item => {
                warn!("{item} is not implemented yet");
                Control::None
            }
        }
    }

    fn get_menu_hint(&self) -> Option<&str> {
        get_menu_hint(self.menu.get(self.menu.state.selected()?)?)
    }
}

impl Menu for ConfigMenu {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let area = utils::menu_block("Settings", area, buf);
        let len = self.menu.len();
        let area = utils::render_hint(area, buf, len, self.get_menu_hint());
        self.menu.render(area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        self.menu
            .key_event(backend, event)
            .to_control(|i| self.menu_exec(i))
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.menu.mouse_event(backend)
    }
}
