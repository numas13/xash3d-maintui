mod screen;
mod state;

pub mod sound;
pub mod symbols;
pub mod utils;

use core::ffi::c_int;

use alloc::{boxed::Box, vec::Vec};
use csz::CStrThin;
use ratatui::prelude::*;
use xash3d_ratatui::{XashBackend, XashTerminal};
use xash3d_ui::{
    color::RGBA,
    engine::{ActiveMenu, net::netadr_s},
    export::UnsyncGlobal,
};

use crate::{
    export::Dll,
    i18n,
    input::{Key, KeyEvent, Modifier},
    prelude::*,
    strings::{self, Localize},
    widgets::{ConfirmPopup, ConfirmResult, WidgetMut},
};

pub use self::{screen::Screen, state::State};

pub enum Control {
    None,
    Back,
    BackHide,
    BackMain,
    BackMainHide,
    Hide,
    Next(Box<dyn Menu>),
    Console,
    GrabInput(bool),
    QuitPopup,
}

impl Control {
    pub fn next(menu: impl Menu + 'static) -> Control {
        Self::Next(Box::new(menu))
    }
}

#[allow(unused_variables)]
pub trait Menu {
    fn vid_init(&mut self) {}
    fn active(&mut self) {}
    fn on_menu_hide(&mut self) {}
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen);
    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        Control::None
    }
    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        false
    }
    fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {}
    fn reset_ping(&mut self) {}
    fn add_touch_button_to_list(
        &mut self,
        name: &CStrThin,
        texture: &CStrThin,
        command: &CStrThin,
        color: RGBA,
        flags: c_int,
    ) {
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Touch {
    Start(Position),
    Active(Position),
    Stop,
}

impl Touch {
    fn is_active(&self) -> bool {
        matches!(self, Self::Start(_) | Self::Active(_))
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Focus {
    Main,
    QuitPopup,
}

pub struct Ui {
    engine: UiEngineRef,
    terminal: XashTerminal,
    history: Vec<Box<dyn Menu>>,
    active: bool,
    grab_input: bool,
    modifier: Modifier,
    focus: Focus,
    touch_start: f32,
    touch: Touch,
    emulated_wheel: Option<Position>,
    quit_popup: Option<ConfirmPopup>,
}

impl Ui {
    pub fn new(engine: UiEngineRef) -> Self {
        strings::init();

        // TODO: helper macro
        unsafe extern "C" fn cmd_fg() {
            unsafe { Dll::global_assume_init_ref() }
                .ui_mut()
                .activate_console(false);
        }
        engine.add_command(c"fg", cmd_fg).unwrap();

        Self {
            engine,
            history: vec![],
            terminal: XashTerminal::new(engine),
            active: false,
            grab_input: false,
            modifier: Modifier::default(),
            focus: Focus::Main,
            touch_start: 0.0,
            touch: Touch::Stop,
            emulated_wheel: None,
            quit_popup: None,
        }
    }

    pub fn vid_init(&mut self) -> bool {
        let globals = &self.engine.globals;
        let width = globals.screen_width();
        let height = globals.screen_height();
        self.terminal.resize(width, height);
        for menu in &mut self.history {
            menu.vid_init();
        }
        true
    }

    fn activate_console(&mut self, active: bool) {
        self.active = !active;
        if active {
            self.engine.set_key_dest(ActiveMenu::Console);
        } else {
            self.engine.set_key_dest(ActiveMenu::Menu);
        }
    }

    fn back(&mut self) -> bool {
        if self.history.len() > 1 {
            self.history.pop();
            if let Some(menu) = self.history.last_mut() {
                menu.active();
            }
            true
        } else {
            false
        }
    }

    fn quit(&mut self) {
        while let Some(mut i) = self.history.pop() {
            i.on_menu_hide();
        }
        self.engine.client_cmd(c"quit");
    }

    fn back_main(&mut self) {
        while self.history.len() > 1 {
            self.history.pop();
        }
        sound::switch_menu();
        self.history[0].active();
    }

    pub fn set_active_menu(&mut self, active: bool) {
        trace!("Ui::set_active_menu({active:?})");

        self.active = active;
        if active {
            self.engine.set_key_dest(ActiveMenu::Menu);
            sound::switch_menu();
        } else {
            if let Some(menu) = self.history.last_mut() {
                menu.on_menu_hide();
            }
            self.engine.set_key_dest(ActiveMenu::Game);
        }
    }

    fn hide(&mut self) {
        self.set_active_menu(false);
    }

    pub fn is_visible(&self) -> bool {
        // trace!("Ui::is_visible()");
        self.active
    }

    fn key_event_menu(&mut self, event: KeyEvent) {
        if let Some(menu) = self.history.last_mut() {
            let control = menu.key_event(self.terminal.backend(), event);
            if self.grab_input && !matches!(control, Control::None) {
                self.grab_input = false;
            }
            match control {
                Control::None => {}
                Control::Back => {
                    if self.back() {
                        sound::switch_menu();
                    }
                }
                Control::BackHide => {
                    self.back();
                    self.hide();
                }
                Control::BackMain => {
                    self.back_main();
                }
                Control::BackMainHide => {
                    self.back_main();
                    self.hide();
                }
                Control::Hide => {
                    self.hide();
                }
                Control::Next(mut menu) => {
                    menu.active();
                    self.history.push(menu);
                    sound::switch_menu();
                }
                Control::Console => self.activate_console(true),
                Control::GrabInput(enabled) => self.grab_input = enabled,
                Control::QuitPopup => {
                    self.change_state_quit();
                }
            }
        }
    }

    fn change_state_quit(&mut self) {
        if let Some(menu) = self.history.last_mut() {
            menu.on_menu_hide();
        }
        self.focus = Focus::QuitPopup;
        sound::select_item();
    }

    fn change_state_deny(&mut self) {
        self.focus = Focus::Main;
        sound::deny();
    }

    fn handle_touch(&mut self) {
        let (Touch::Start(prev) | Touch::Active(prev)) = self.touch else {
            return;
        };
        let backend = self.terminal.backend();
        let new = backend.cursor_position_in_pixels();
        if matches!(self.touch, Touch::Start(_)) {
            let key = Key::TouchStart(backend.mouse_to_cursor(prev));
            let event = KeyEvent::new_touch(self.modifier, key);
            self.key_event_menu(event);
        }
        if prev != new {
            let x = prev.x as i32 - new.x as i32;
            let y = prev.y as i32 - new.y as i32;
            let key = Key::Touch(x, y);
            let event = KeyEvent::new_touch(self.modifier, key);
            self.key_event_menu(event);
        }
        self.touch = Touch::Active(new);
    }

    fn handle_emulated_wheel_event(&mut self, pos: Position) {
        use xash3d_ui::consts::keys::*;

        let backend = self.terminal.backend();
        let cursor = backend.cursor_position();
        if cursor == pos {
            return;
        }
        self.emulated_wheel = Some(cursor);

        let Some(menu) = self.history.last_mut() else {
            return;
        };

        if pos.x != cursor.x {
            let key = if pos.x > cursor.x {
                Key::MouseWheelLeft(1)
            } else {
                Key::MouseWheelRight(1)
            };
            let event = KeyEvent::with_key(0, self.modifier, true, key);
            menu.key_event(backend, event);
        }

        if pos.y != cursor.y {
            let (raw, key) = if pos.y > cursor.y {
                (K_MWHEELDOWN, Key::MouseWheelDown(1))
            } else {
                (K_MWHEELUP, Key::MouseWheelUp(1))
            };
            let event = KeyEvent::with_key(raw, self.modifier, true, key);
            menu.key_event(backend, event);
        }
    }

    pub fn redraw(&mut self, _time: f32) {
        if !self.active {
            return;
        }

        if self.history.is_empty() {
            // XXX: init here bacause ui_language cvar needed for localization is not ready
            // in Ui::init() and Ui::vid_init()
            self.history.push(crate::menu::main());
            self.quit_popup = Some(ConfirmPopup::with_title(
                i18n::all::QUIT_POPUP_TITLE.localize(),
                i18n::all::QUIT_POPUP_BODY.localize(),
            ));
        }

        self.terminal.backend_mut().draw_background();
        if let Some(menu) = self.history.last_mut() {
            self.terminal.draw(|area, buffer, backend| {
                let screen = Screen::new(backend);

                menu.draw(area, buffer, &screen);

                if self.focus == Focus::QuitPopup {
                    let popup = self.quit_popup.as_mut().unwrap();
                    popup.render(area, buffer, &screen);
                }
            });
        }
    }

    pub fn key_event(&mut self, key: c_int, down: bool) {
        // trace!("Ui::key_event({key}, {down})");
        let event = KeyEvent::new(key as u8, self.modifier, down);
        let key = event.key();

        if self.grab_input {
            if down {
                self.key_event_menu(event);
            }
            return;
        }

        match key {
            Key::Ctrl => self.modifier.ctrl = down,
            Key::Shift => self.modifier.shift = down,
            Key::Alt => self.modifier.alt = down,
            _ => {
                if key == Key::Mouse(0) {
                    let backend = self.terminal.backend();
                    if event.is_down() {
                        self.touch_start = self.engine.globals.system_time_f32();
                        self.touch = Touch::Start(backend.cursor_position_in_pixels());
                        self.emulated_wheel = Some(backend.cursor_position());
                        return;
                    } else {
                        let is_touch_active = self.touch.is_active();
                        self.touch = Touch::Stop;
                        self.emulated_wheel = None;
                        if self.engine.globals.system_time_f32() - self.touch_start >= 0.2 {
                            if is_touch_active {
                                let key = Key::TouchStop(backend.cursor_position_in_pixels());
                                let event = KeyEvent::new_touch(self.modifier, key);
                                self.key_event_menu(event);
                            }
                            return;
                        }
                    }
                } else if event.is_up() {
                    return;
                }

                // pressing escape in the main menu returns back to the game
                if key == Key::Escape && self.history.len() == 1 && self.engine.client_in_game() {
                    self.set_active_menu(false);
                    return;
                }

                match self.focus {
                    Focus::Main => match key {
                        Key::Char(b'q') if event.ctrl() => {
                            self.change_state_quit();
                        }
                        Key::Char(b'z') if event.ctrl() => self.activate_console(true),
                        Key::Char(b'-') if event.ctrl() => {
                            self.terminal.backend_mut().decrease_font_size()
                        }
                        Key::Char(b'=') if event.ctrl() => {
                            self.terminal.backend_mut().increase_font_size()
                        }
                        _ => self.key_event_menu(event),
                    },
                    Focus::QuitPopup => {
                        let popup = self.quit_popup.as_mut().unwrap();
                        match popup.key_event(self.terminal.backend(), event) {
                            ConfirmResult::None => {}
                            ConfirmResult::Cancel => self.change_state_deny(),
                            ConfirmResult::Ok => self.quit(),
                        }
                    }
                }
            }
        }
    }

    pub fn mouse_move(&mut self, x: c_int, y: c_int) {
        //trace!("Ui::mouse_move({x}, {y})");
        let pos = (x.max(0) as u16, y.max(0) as u16).into();
        if self.terminal.backend_mut().set_cursor_position(pos) {
            match self.focus {
                Focus::Main => {
                    self.handle_touch();

                    match self.emulated_wheel {
                        Some(pos) => self.handle_emulated_wheel_event(pos),
                        None => {
                            if let Some(menu) = self.history.last_mut() {
                                menu.mouse_event(self.terminal.backend());
                            }
                        }
                    }
                }
                Focus::QuitPopup => {
                    self.quit_popup
                        .as_mut()
                        .unwrap()
                        .mouse_event(self.terminal.backend());
                }
            }
        }
    }

    pub fn add_touch_button_to_list(
        &mut self,
        name: &CStrThin,
        texture: &CStrThin,
        command: &CStrThin,
        color: RGBA,
        flags: c_int,
    ) {
        if let Some(menu) = self.history.last_mut() {
            menu.add_touch_button_to_list(name, texture, command, color, flags);
        }
    }

    pub fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {
        for i in &mut self.history {
            i.add_server_to_list(addr, info);
        }
    }

    pub fn reset_ping(&mut self) {
        trace!("Ui::reset_ping");
        if let Some(menu) = self.history.last_mut() {
            menu.reset_ping();
        }
    }
}
