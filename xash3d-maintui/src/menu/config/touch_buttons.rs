use core::ffi::c_int;

use alloc::vec::Vec;
use compact_str::{CompactString, ToCompactString};
use csz::CStrThin;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{color::RGBA, engine};

use crate::{
    input::{Key, KeyEvent},
    menu::define_menu_items,
    ui::{sound, utils, Control, Menu, Screen, State},
    widgets::{ConfirmPopup, ConfirmResult, List, ListPopup, SelectResult, WidgetMut},
};

define_menu_items! {
    MENU_BACK = "Back", "Go back to the settings menu.";
    MENU_RESET = "Reset", "Reset all buttons to default values.";
    // TODO: create a new button
}

const CONTEXT_CANCEL: &str = "Cancel";
const CONTEXT_REMOVE: &str = "Remove";

#[allow(dead_code)]
struct Button {
    name: CompactString,
    texture: CompactString,
    command: CompactString,
    color: RGBA,
    flags: c_int,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Menu,
    List,
    DeletePopoup(usize),
    ContextMenu(usize),
}

pub struct TouchButtonsConfig {
    state: State<Focus>,
    menu: List,
    list: List,
    remove_popup: ConfirmPopup,
    context_menu: ListPopup,
    buttons: Vec<Button>,
}

impl TouchButtonsConfig {
    pub fn new() -> Self {
        let mut menu = List::new_first([MENU_BACK, MENU_RESET]);
        menu.set_bindings([(Key::Char(b'r'), MENU_RESET), (Key::Char(b'b'), MENU_BACK)]);
        Self {
            state: State::default(),
            menu,
            list: List::empty(),
            remove_popup: ConfirmPopup::new("Do you want to remove button?"),
            context_menu: ListPopup::new("Button", [CONTEXT_CANCEL, CONTEXT_REMOVE]),
            buttons: Default::default(),
        }
    }

    fn clear(&mut self) {
        self.buttons.clear();
        self.list.clear();
        self.list.state.select(None);
    }

    fn load_list(&mut self) {
        self.clear();
        engine().client_cmd(c"touch_list");
    }

    fn load_defaults(&mut self) {
        let engine = engine();
        engine.client_cmd(c"touch_removeall");
        engine.client_cmd(c"touch_loaddefaults");
        self.load_list();
    }

    fn remove_button(&mut self, i: usize) {
        self.state.set(Focus::List);
        let name = &self.list[i];
        engine().client_cmdf(format_args!("touch_removebutton \"{name}\""));
        sound::confirm();
        self.load_list();
    }

    fn hint(&self) -> Option<&str> {
        match self.state.focus() {
            Focus::Menu => {
                let selected = self.menu.state.selected()?;
                get_menu_hint(self.menu.get(selected)?)
            }
            Focus::List => Some("Open menu to change button settings."),
            _ => None,
        }
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_BACK => return Control::Back,
            MENU_RESET => self.load_defaults(),
            item => warn!("{item} is not implemented yet"),
        }
        Control::None
    }

    fn list_exec(&mut self, i: usize) -> Control {
        let name = &self.list[i];
        let Some(button) = self.buttons.iter().find(|i| i.name == name) else {
            return Control::None;
        };
        warn!("button {} is not implemented yet", button.name);
        Control::None
    }
}

impl Menu for TouchButtonsConfig {
    fn active(&mut self) {
        self.load_list();
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let len = self.menu.len() + self.list.len();
        let area = utils::main_block("Touch buttons settings", area, buf);
        let area = utils::render_hint(area, buf, len, self.hint());
        let [menu_area, list_area] = Layout::vertical([
            Constraint::Length(self.menu.len() as u16),
            Constraint::Percentage(100),
        ])
        .areas(area);
        self.menu.render(menu_area, buf, screen);
        self.list.render(list_area, buf, screen);

        match self.state.focus() {
            Focus::DeletePopoup(..) => self.remove_popup.render(area, buf, screen),
            Focus::ContextMenu(..) => self.context_menu.render(area, buf, screen),
            _ => {}
        }
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match self.state.focus() {
            Focus::Menu => match self.menu.key_event(backend, event) {
                SelectResult::Down => {
                    self.menu.state.select(None);
                    self.list.state.select_first();
                    self.state.next(Focus::List);
                }
                SelectResult::Ok(i) => return self.menu_exec(i),
                SelectResult::Cancel => return Control::Back,
                _ => {}
            },
            Focus::List => {
                if let Some(i) = self.menu.match_binding(key) {
                    return self.menu_exec(i);
                }
                match key {
                    Key::Char(b'd') | Key::Delete => {
                        if let Some(i) = self.list.state.selected() {
                            self.state.select(Focus::DeletePopoup(i));
                        }
                    }
                    _ => match self.list.key_event(backend, event) {
                        SelectResult::Up => {
                            self.menu.state.select_last();
                            self.list.state.select(None);
                            self.state.prev(Focus::Menu);
                        }
                        SelectResult::ContextMenu(i) => self.state.select(Focus::ContextMenu(i)),
                        SelectResult::Ok(i) => return self.list_exec(i),
                        SelectResult::Cancel => return Control::Back,
                        _ => {}
                    },
                }
            }
            Focus::DeletePopoup(i) => match self.remove_popup.key_event(backend, event) {
                ConfirmResult::None => {}
                ConfirmResult::Cancel => self.state.deny_default(),
                ConfirmResult::Ok => self.remove_button(*i),
            },
            Focus::ContextMenu(i) => match self.context_menu.key_event(backend, event) {
                SelectResult::Ok(j) => match self.context_menu.get(j) {
                    Some(CONTEXT_CANCEL) => self.state.cancel_default(),
                    Some(CONTEXT_REMOVE) => self.state.select(Focus::DeletePopoup(*i)),
                    _ => {}
                },
                SelectResult::Cancel => self.state.cancel_default(),
                _ => {}
            },
        }
        Control::None
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        match self.state.focus() {
            Focus::Menu | Focus::List => {
                if self.menu.mouse_event(backend) {
                    self.list.state.select(None);
                    self.state.set(Focus::Menu);
                    true
                } else if self.list.mouse_event(backend) {
                    self.menu.state.select(None);
                    self.state.set(Focus::List);
                    true
                } else {
                    false
                }
            }
            Focus::DeletePopoup(..) => self.remove_popup.mouse_event(backend),
            Focus::ContextMenu(..) => self.context_menu.mouse_event(backend),
        }
    }

    fn add_touch_button_to_list(
        &mut self,
        name: &CStrThin,
        texture: &CStrThin,
        command: &CStrThin,
        color: RGBA,
        flags: c_int,
    ) {
        let button = Button {
            name: name.to_compact_string(),
            texture: texture.to_compact_string(),
            command: command.to_compact_string(),
            color,
            flags,
        };
        self.list.push(button.name.clone());
        self.buttons.push(button);
    }
}
