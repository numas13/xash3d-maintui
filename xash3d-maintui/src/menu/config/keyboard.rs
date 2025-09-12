use core::{
    ffi::{c_int, CStr},
    str,
};

use compact_str::{CompactString, ToCompactString};
use csz::CStrArray;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use xash3d_ratatui::XashBackend;
use xash3d_ui::{
    parser::{TokenError, Tokens},
    prelude::*,
};

use crate::{
    input::{Key, KeyEvent},
    strings::Localize,
    ui::{sound, utils, Control, Menu, Screen, State},
    widgets::{List, MyTable, SelectResult, WidgetMut},
};

mod i18n {
    pub use crate::i18n::{all::*, menu::config_keyboard::*};
}

const MAX_KEYS: usize = 256;

const KEYBOARD_ACTION_LIST_PATH: &CStr = c"gfx/shell/kb_act.lst";
const KEYBOARD_DEFAULT_LIST_PATH: &CStr = c"gfx/shell/kb_def.lst";

const MENU_BACK: &str = i18n::BACK;
const MENU_RESET: &str = i18n::RESET;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Menu,
    Table,
    EditPopup(usize),
}

enum Item {
    Text(CompactString),
    Binding {
        bind: CompactString,
        name: CompactString,
        keys: [CompactString; 2],
    },
}

pub struct Controls {
    state: State<Focus>,
    menu: List,
    table: MyTable<Item>,
}

impl Controls {
    pub fn new() -> Self {
        let mut menu = List::new_first([MENU_BACK, MENU_RESET]);
        menu.set_bindings([(Key::Char(b'b'), MENU_BACK), (Key::Char(b'r'), MENU_RESET)]);
        let mut this = Self {
            state: State::default(),
            menu,
            table: Default::default(),
        };
        this.load_keys();
        this
    }

    fn get_keys(&self, name: &str) -> [CompactString; 2] {
        let engine = engine();
        let mut keys = [-1; 2];
        let bindings = (0..MAX_KEYS as c_int).filter(|i| match engine.key_get_binding(*i) {
            Some(s) => s.to_bytes().eq_ignore_ascii_case(name.as_bytes()),
            None => false,
        });
        for (i, keynum) in bindings.take(2).enumerate() {
            keys[i] = keynum;
        }
        // swap keys if both is set
        if keys.iter().all(|i| *i != -1) {
            keys.reverse();
        }

        let keynum_to_str = |keynum| {
            if keynum != -1 {
                let mut buffer = CStrArray::<128>::new();
                match engine.keynum_to_str_buffer(keynum, &mut buffer) {
                    Ok(s) => return s.to_compact_string(),
                    Err(_) => error!("failed to get string for key({keynum})"),
                }
            }
            CompactString::default()
        };
        [keynum_to_str(keys[0]), keynum_to_str(keys[1])]
    }

    fn load_file(path: &CStr, mut f: impl FnMut(&str, &str)) {
        let file = engine().load_file(path).unwrap();
        let Ok(data) = str::from_utf8(file.as_slice()) else {
            error!("file does not contain a valid UTF-8 data, {path:?}");
            return;
        };

        let mut parse = |data| -> Result<(), TokenError> {
            let mut tokens = Tokens::new(data);
            while let Some(token) = tokens.next() {
                f(token?, tokens.parse()?);
            }
            Ok(())
        };
        if let Err(e) = parse(data) {
            error!("Failed to parse {path:?}: {e}");
        }
    }

    fn load_keys(&mut self) {
        self.table.clear();
        Self::load_file(KEYBOARD_ACTION_LIST_PATH, |bind, name| {
            let name = name.localize().into();
            if bind == "blank" {
                self.table.push(Item::Text(name));
            } else {
                let keys = self.get_keys(bind);
                self.table.push(Item::Binding {
                    bind: bind.into(),
                    name,
                    keys,
                });
            }
        });
    }

    fn reset_keys(&mut self) {
        let engine = engine();
        engine.client_cmd_now(c"unbindall");
        Self::load_file(KEYBOARD_DEFAULT_LIST_PATH, |bind, mut name| {
            if name == "\\\\" {
                name = "\\";
            }
            engine.client_cmd_now(format_args!("bind \"{bind}\" \"{name}\""));
        });
        self.load_keys();
    }

    fn unbind_command(bind: &str) {
        let engine = engine();
        for i in 0..MAX_KEYS as c_int {
            if let Some(s) = engine.key_get_binding(i) {
                if s.to_str().is_ok_and(|s| s == bind) {
                    engine.key_set_binding(i, c"");
                }
            }
        }
    }

    fn unbind_all(&mut self) {
        engine().client_cmd_now(c"unbindall");
        self.load_keys();
    }

    fn bind_key(&mut self, index: usize, raw: u8) {
        let Some(Item::Binding { bind, keys, .. }) = self.table.get(index) else {
            return;
        };
        let engine = engine();
        let mut buffer = CStrArray::<128>::new();
        if let Ok(s) = engine.keynum_to_str_buffer(raw as c_int, &mut buffer) {
            if !keys[1].is_empty() {
                Self::unbind_command(bind);
            }
            engine.client_cmd_now(format_args!("bind \"{s}\" \"{bind}\""));
        } else {
            error!("failed to bind key \"{bind}\"");
        }
        self.load_keys();
        sound::confirm();
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_BACK => return Control::Back,
            MENU_RESET => self.reset_keys(),
            item => warn!("unimplemented: exec {item}"),
        }
        Control::None
    }

    fn table_exec(&mut self, i: usize) -> Control {
        if let Some(Item::Binding { .. }) = self.table.get(i) {
            self.state.set(Focus::EditPopup(i));
            return Control::GrabInput(true);
        }
        Control::None
    }

    fn draw_menu(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_style(utils::main_block_border_style());
        let menu_area = block.inner(area);
        block.render(area, buf);
        self.menu.render(menu_area, buf, screen);
    }

    fn draw_table(&mut self, area: Rect, buf: &mut Buffer) {
        let header = Row::new([
            i18n::COLUMN_ACTION.localize(),
            i18n::COLUMN_KEY.localize(),
            i18n::COLUMN_KEY_ALT.localize(),
        ]);
        let table = Table::default()
            .header(header.style(Style::new().bold()))
            .widths([
                Constraint::Fill(3),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]);

        let focused = matches!(self.state.focus(), Focus::Table);
        self.table.draw(area, buf, table, focused, |i| {
            let cells = match i {
                Item::Text(text) => [Cell::new(text.as_str()), Cell::new(""), Cell::new("")],
                Item::Binding { name, keys, .. } => [
                    Cell::new(name.as_str()),
                    Cell::new(keys[0].as_str()),
                    Cell::new(keys[1].as_str()),
                ],
            };
            Some(Row::new(cells))
        });
    }

    fn draw_popup(&mut self, area: Rect, buf: &mut Buffer) {
        if !matches!(self.state.focus(), Focus::EditPopup(_)) {
            return;
        }
        let block = utils::popup_block("");
        let text = i18n::PRESS_KEY.localize();
        let line = Paragraph::new(text)
            .block(block)
            .style(Style::new().black().bold().on_gray());
        let width = 2 + text.len() as u16;
        let area = utils::centered_rect_fixed(width, 3, area);
        line.render(area, buf);
    }
}

impl Menu for Controls {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let title = i18n::TITLE;
        let [menu_area, table_area] = Layout::vertical([
            Constraint::Length(self.menu.len() as u16 + 1),
            Constraint::Percentage(100),
        ])
        .areas(utils::main_block(title, area, buf));

        self.draw_menu(menu_area, buf, screen);
        self.draw_table(table_area, buf);
        self.draw_popup(area, buf);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match self.state.focus() {
            Focus::Menu => match self.menu.key_event(backend, event) {
                SelectResult::Ok(i) => return self.menu_exec(i),
                SelectResult::Down => {
                    self.menu.state.select(None);
                    self.table.state.select_first();
                    self.state.next(Focus::Table);
                }
                SelectResult::Cancel => return Control::Back,
                _ => {}
            },
            Focus::Table => {
                if let Some(i) = self.menu.match_binding(key) {
                    return self.menu_exec(i);
                }
                match key {
                    Key::Delete if event.shift() => self.unbind_all(),
                    Key::Delete => {
                        let selected = self.table.state.selected();
                        let item = selected.and_then(|i| self.table.get_mut(i));
                        if let Some(Item::Binding { bind, keys, .. }) = item {
                            Self::unbind_command(bind);
                            for i in keys {
                                i.clear();
                            }
                        }
                    }
                    _ => match self.table.key_event(backend, event) {
                        SelectResult::Ok(i) => return self.table_exec(i),
                        SelectResult::Up => {
                            self.table.state.select(None);
                            self.menu.state.select_last();
                            self.state.prev(Focus::Menu);
                        }
                        SelectResult::Cancel => return Control::Back,
                        _ => {}
                    },
                }
            }
            Focus::EditPopup(i) => {
                match key {
                    Key::Escape => sound::deny2(),
                    _ => self.bind_key(*i, event.raw()),
                }
                self.state.set(Focus::Table);
                return Control::GrabInput(false);
            }
        }
        Control::None
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        if let Focus::Menu | Focus::Table = self.state.focus() {
            if self.menu.mouse_event(backend) {
                self.state.set(Focus::Menu);
                return true;
            } else if self.table.mouse_event(backend) {
                self.menu.state.select(None);
                self.state.set(Focus::Table);
                return true;
            }
        }
        false
    }
}
