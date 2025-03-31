mod screen;
mod state;

pub mod sound;
pub mod symbols;
pub mod utils;

use std::{
    ffi::{c_char, c_int, c_uchar},
    slice,
};

use csz::CStrThin;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_shared::{color::RGBA, raw::netadr_s};
use xash3d_ui::{engine, globals, ActiveMenu};
use xash3d_utils::macros::unimpl;

use crate::{
    export::{Api, MenuApi, UiFunctions, UiFunctionsExtended},
    input::{Key, KeyEvent, Modifier},
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
    terminal: Terminal<XashBackend>,
    history: Vec<Box<dyn Menu>>,
    active: bool,
    grab_input: bool,
    modifier: Modifier,
    focus: Focus,
    touch_start: f32,
    touch: Touch,
    emulated_wheel: Option<Position>,
    quit_popup: ConfirmPopup,
}

impl Ui {
    pub fn new() -> Ui {
        let backend = XashBackend::new();
        let terminal = Terminal::new(backend).expect("failed to create terminal");

        // TODO: helper macro
        unsafe extern "C" fn cmd_fg() {
            Api::global().activate_console(false);
        }
        engine().add_command(c"fg", Some(cmd_fg));

        Ui {
            history: vec![crate::menu::main()],
            terminal,
            active: false,
            grab_input: false,
            modifier: Modifier::default(),
            focus: Focus::Main,
            touch_start: 0.0,
            touch: Touch::Stop,
            emulated_wheel: None,
            quit_popup: ConfirmPopup::new("Do you want to exit?"),
        }
    }

    fn activate_console(&mut self, active: bool) {
        self.active = !active;
        if active {
            engine().set_key_dest(ActiveMenu::Console);
        } else {
            engine().set_key_dest(ActiveMenu::Menu);
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
        engine().client_cmd(c"quit");
    }

    fn back_main(&mut self) {
        while self.history.len() > 1 {
            self.history.pop();
        }
        sound::switch_menu();
        self.history[0].active();
    }

    fn hide(&mut self) {
        self.set_active_menu(false);
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
}

impl UiFunctions for Ui {
    fn vid_init(&mut self) -> c_int {
        trace!("Ui::vid_init()");

        let globals = globals();
        let backend = self.terminal.backend_mut();
        backend.resize(globals.scrWidth as usize, globals.scrHeight as usize);

        self.terminal.clear().unwrap();

        0
    }

    fn shutdown(&mut self) {
        trace!("Ui::shutdown()");
    }

    fn redraw(&mut self, _time: f32) {
        if !self.active {
            return;
        }

        let backend = self.terminal.backend_mut();
        backend.draw_background();

        let Some(menu) = self.history.last_mut() else {
            return;
        };
        let screen = Screen::new(backend);
        self.terminal
            .draw(|frame| {
                menu.draw(frame.area(), frame.buffer_mut(), &screen);
                if self.focus == Focus::QuitPopup {
                    self.quit_popup
                        .render(frame.area(), frame.buffer_mut(), &screen);
                }
            })
            .unwrap();

        // TODO: remove when cell cache will be implemented
        self.terminal.clear().unwrap();
    }

    fn key_event(&mut self, key: c_int, down: c_int) {
        // trace!("Ui::key_event({key}, {down})");
        let event = KeyEvent::new(key as u8, self.modifier, down != 0);
        let key = event.key();

        if self.grab_input {
            if down == 1 {
                self.key_event_menu(event);
            }
            return;
        }

        match key {
            Key::Ctrl => self.modifier.ctrl = down != 0,
            Key::Shift => self.modifier.shift = down != 0,
            Key::Alt => self.modifier.alt = down != 0,
            _ => {
                if key == Key::Mouse(0) {
                    let backend = self.terminal.backend();
                    if event.is_down() {
                        self.touch_start = globals().time;
                        self.touch = Touch::Start(backend.cursor_position_in_pixels());
                        self.emulated_wheel = Some(backend.cursor_position());
                        return;
                    } else {
                        let is_touch_active = self.touch.is_active();
                        self.touch = Touch::Stop;
                        self.emulated_wheel = None;
                        if globals().time - self.touch_start >= 0.2 {
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

                if key == Key::Escape {
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
                        match self.quit_popup.key_event(self.terminal.backend(), event) {
                            ConfirmResult::None => {}
                            ConfirmResult::Cancel => self.change_state_deny(),
                            ConfirmResult::Ok => self.quit(),
                        }
                    }
                }
            }
        }
    }

    fn mouse_move(&mut self, x: c_int, y: c_int) {
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
                    self.quit_popup.mouse_event(self.terminal.backend());
                }
            }
        }
    }

    fn set_active_menu(&mut self, active: bool) {
        trace!("Ui::set_active_menu({active:?})");

        self.active = active;
        if active {
            engine().set_key_dest(ActiveMenu::Menu);
            sound::switch_menu();
        } else {
            if let Some(menu) = self.history.last_mut() {
                menu.on_menu_hide();
            }
            engine().set_key_dest(ActiveMenu::Game);
        }
    }

    fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {
        if let Some(menu) = self.history.last_mut() {
            menu.add_server_to_list(addr, info);
        }
    }

    fn get_cursor_pos(&mut self, x: *mut c_int, y: *mut c_int) {
        trace!("Ui::get_cursor_pos({x:?}, {y:?})");
    }

    fn set_cursor_pos(&mut self, x: c_int, y: c_int) {
        trace!("Ui::set_cursor_pos({x}, {y})");
    }

    fn show_cursor(&mut self, show: c_int) {
        trace!("Ui::show_cursor({show})");
    }

    fn char_event(&mut self, key: c_int) {
        trace!("Ui::char_event({key})");
    }

    fn mouse_in_rect(&mut self) -> c_int {
        trace!("Ui::mouse_in_rect()");
        0
    }

    fn is_visible(&mut self) -> c_int {
        // trace!("Ui::is_visible()");
        self.active as c_int
    }

    fn credits_active(&mut self) -> c_int {
        trace!("Ui::credits_active()");
        0
    }

    fn final_credits(&mut self) {
        trace!("Ui::final_credits()");
    }
}

#[allow(unused_variables)]
impl UiFunctionsExtended for Ui {
    fn add_touch_button_to_list(
        &mut self,
        name: *const c_char,
        texture: *const c_char,
        command: *const c_char,
        color: *mut c_uchar,
        flags: c_int,
    ) {
        let name = unsafe { CStrThin::from_ptr(name) };
        let texture = unsafe { CStrThin::from_ptr(texture) };
        let command = unsafe { CStrThin::from_ptr(command) };
        let color = unsafe { slice::from_raw_parts(color as *const u8, 4) };
        let color = RGBA::new(color[0], color[1], color[2], color[3]);
        if let Some(menu) = self.history.last_mut() {
            menu.add_touch_button_to_list(name, texture, command, color, flags);
        }
    }

    fn reset_ping(&mut self) {
        trace!("Ui::reset_ping");
        if let Some(menu) = self.history.last_mut() {
            menu.reset_ping();
        }
    }

    fn show_connection_warning(&mut self) {
        unimpl!("show_connection_warning");
    }

    fn show_update_dialog(&mut self, prefer_store: c_int) {
        unimpl!("show_update_dialog");
    }

    fn show_message_box(&mut self, text: &CStrThin) {
        unimpl!("show_message_box");
    }

    fn connection_progress_disconnect(&mut self) {
        unimpl!("connection_progress_disconnect");
    }

    fn connection_progress_download(
        &mut self,
        file_name: &CStrThin,
        server_name: &CStrThin,
        current: c_int,
        total: c_int,
        comment: &CStrThin,
    ) {
        unimpl!("connection_progress_download");
    }

    fn connection_process_download_end(&mut self) {
        unimpl!("connection_process_download_end");
    }

    fn connection_progress_precache(&mut self) {
        unimpl!("connection_progress_precache");
    }

    fn connection_progress_connect(&mut self, server: &CStrThin) {
        unimpl!("connection_progress_connect");
    }

    fn connection_progress_change_level(&mut self) {
        unimpl!("connection_progress_change_level");
    }

    fn connection_progress_parse_server_info(&mut self, server: &CStrThin) {
        unimpl!("connection_progress_parse_server_info");
    }
}
