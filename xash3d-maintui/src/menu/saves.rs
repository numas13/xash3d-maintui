use std::{
    ffi::{CStr, CString},
    path::Path,
    str,
};

use libc::c_int;
use ratatui::{
    prelude::*,
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table},
};
use xash3d_ratatui::XashBackend;
use xash3d_shared::consts::{CS_SIZE, CS_TIME};
use xash3d_ui::engine;

use crate::{
    input::{Key, KeyEvent},
    strings::Localize,
    ui::{sound, utils, Control, Menu, Screen, State},
    widgets::{
        ConfirmPopup, ConfirmResult, Image, List, ListPopup, MyTable, SelectResult, WidgetMut,
    },
};

mod i18n {
    pub use crate::i18n::{all::*, menu::save::*};
}

const MENU_BACK: &str = i18n::BACK;

const CONTEXT_CANCEL: &str = i18n::CANCEL;
const CONTEXT_DELETE: &str = i18n::DELETE_SAVE;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    Menu,
    #[default]
    Table,
    DeletePopoup(usize),
    ContextMenu(usize),
}

#[derive(Default)]
struct SaveInfo {
    filename: String,
    comment: String,
    datetime: String,
}

struct SavePreview {
    filename: String,
    picture: c_int,
}

impl SavePreview {
    fn new(filename: String) -> Self {
        let path = format!("save/{filename}.bmp");
        let picture = engine().pic_load(path, None, 0);
        Self { filename, picture }
    }
}

pub struct SavesMenu {
    state: State<Focus>,
    menu: List,
    table: MyTable<SaveInfo>,
    is_save: bool,
    delete_popup: ConfirmPopup,
    context_menu: ListPopup,
    preview: Option<SavePreview>,
}

impl SavesMenu {
    pub fn new(is_save: bool) -> Self {
        let mut menu = List::new([MENU_BACK]);
        menu.set_bindings([(Key::Char(b'b'), MENU_BACK)]);
        let delete_popup = ConfirmPopup::with_title(
            i18n::DELETE_POPUP_TITLE.localize(),
            i18n::DELETE_POPUP_BODY.localize(),
        );
        let context_menu = ListPopup::new(
            i18n::CONTEXT_TITLE.localize(),
            [CONTEXT_CANCEL, CONTEXT_DELETE],
        );
        Self {
            state: State::default(),
            menu,
            table: MyTable::new_first(),
            is_save,
            delete_popup,
            context_menu,
            preview: None,
        }
    }

    fn update_list(&mut self) {
        self.table.clear();

        if self.is_save {
            self.table.push(SaveInfo {
                filename: String::from("new"),
                comment: i18n::NEW_SAVE.localize().to_string(),
                datetime: i18n::NOW.localize().to_string(),
            });
        }

        let engine = engine();
        let filenames = engine.get_files_list(c"save/*.sav", true);
        for i in filenames.iter() {
            let mut buf = [0; 256];
            if !engine.get_save_comment(i, &mut buf) {
                // TODO:
                continue;
            }

            let Ok(filename) = i.to_str() else { continue };
            let Some(filename) = Path::new(filename).file_stem() else {
                continue;
            };

            let mut comment = String::with_capacity(CS_SIZE);
            let mut title = &buf[..CS_SIZE - 1];

            // handle `[auto]`, `[quick]`, etc
            if title.starts_with(b"[") {
                if let Some(offset) = title.iter().position(|i| *i == b']') {
                    if let Ok(s) = str::from_utf8(&title[1..offset]) {
                        comment.push('[');
                        comment.push_str(s);
                        comment.push(']');
                        comment.push(' ');
                    }
                    title = &title[offset + 1..];
                }
            }

            let end = if title.starts_with(b"#") {
                let i = title.iter().position(|i| i.is_ascii_whitespace());
                i.unwrap_or(title.len())
            } else {
                let i = title.iter().rev().position(|i| !i.is_ascii_whitespace());
                title.len() - i.unwrap_or(0)
            };
            if let Ok(s) = str::from_utf8(&title[..end]) {
                comment.push_str(s.localize());
            }

            let mut datetime = String::new();
            let date = CStr::from_bytes_until_nul(&buf[CS_SIZE..]);
            if let Ok(s) = date {
                if let Ok(s) = s.to_str() {
                    datetime.push_str(s);
                    datetime.push(' ');
                }
            }
            let time = CStr::from_bytes_until_nul(&buf[CS_SIZE + CS_TIME..]);
            if let Ok(s) = time {
                if let Ok(s) = s.to_str() {
                    datetime.push_str(s);
                }
            }

            self.table.push(SaveInfo {
                filename: filename.to_string_lossy().to_string(),
                comment,
                datetime,
            });
        }

        // Skip fixed entry
        let offset = if self.is_save { 1 } else { 0 };
        self.table[offset..].sort_by(|a, b| a.datetime.cmp(&b.datetime).reverse());

        if self.table.len() == 1 {
            self.table.state.select_first();
        }
    }

    fn delete_save(&mut self, i: usize) {
        self.state.set(Focus::Table);
        let save = self.table.remove(i);
        let cmd = format!("killsave \"{}\"", save.filename);
        if let Ok(cmd) = CString::new(cmd) {
            let eng = engine();
            eng.stop_background_track();
            eng.client_cmd_now(&cmd);
            sound::confirm();
            return;
        }
        sound::deny();
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_BACK => Control::Back,
            item => {
                warn!("{item} is not implemented yet");
                Control::None
            }
        }
    }

    fn table_exec(&mut self, i: usize) -> Control {
        if let Some(save) = self.table.get(i) {
            let cmd = if self.is_save { "save" } else { "load" };
            let cmd = format!("{cmd} \"{}\"", save.filename);
            if let Ok(cmd) = CString::new(cmd) {
                let eng = engine();
                eng.stop_background_track();
                eng.client_cmd(&cmd);
            }
            Control::BackHide
        } else {
            Control::None
        }
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
        let header = Row::new([i18n::TIME.localize(), i18n::SAVE_COMMENT.localize()]);
        let table = Table::default()
            .header(header.style(Style::new().bold()))
            .widths([Constraint::Length(18), Constraint::Min(20)]);

        let focused = !matches!(self.state.focus(), Focus::Menu);
        self.table.draw(area, buf, table, focused, |i| {
            let cells = [
                Cell::new(i.datetime.as_str()),
                Cell::new(i.comment.as_str().localize()),
            ];
            Some(Row::new(cells))
        });
    }

    fn draw_preview(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let block = Block::new()
            .title(i18n::SAVE_PREVIEW.localize())
            .borders(Borders::TOP)
            .border_style(utils::main_block_border_style());
        let inner_area = block.inner(area);
        block.render(area, buf);

        let Some(selected) = self.table.state.selected() else {
            return;
        };
        let Some(save) = self.table.items.get(selected) else {
            return;
        };
        if !matches!(&self.preview, Some(i) if i.filename == save.filename) {
            self.preview = Some(SavePreview::new(save.filename.clone()));
        }
        if let Some(preview) = &self.preview {
            if preview.picture != 0 {
                Image::new(preview.picture).render(inner_area, buf, screen);
            }
        }
    }
}

impl Menu for SavesMenu {
    fn active(&mut self) {
        self.update_list();
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let title = if self.is_save {
            i18n::TITLE_SAVE
        } else {
            i18n::TITLE_LOAD
        };
        let inner_area = utils::main_block(title, area, buf);
        let [menu_area, table_area, preview_area] = Layout::vertical([
            Constraint::Length(self.menu.len() as u16 + 1),
            Constraint::Ratio(2, 3),
            Constraint::Ratio(1, 3),
        ])
        .areas(inner_area);

        self.draw_menu(menu_area, buf, screen);
        self.draw_table(table_area, buf);
        self.draw_preview(preview_area, buf, screen);

        match self.state.focus() {
            Focus::DeletePopoup(..) => self.delete_popup.render(area, buf, screen),
            Focus::ContextMenu(..) => self.context_menu.render(area, buf, screen),
            _ => {}
        }
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
                    Key::Char(b'd') | Key::Delete => {
                        if let Some(i) = self.table.state.selected() {
                            self.state.select(Focus::DeletePopoup(i));
                        }
                    }
                    _ => match self.table.key_event(backend, event) {
                        SelectResult::Ok(i) => return self.table_exec(i),
                        SelectResult::ContextMenu(i) => self.state.select(Focus::ContextMenu(i)),
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
            Focus::DeletePopoup(i) => match self.delete_popup.key_event(backend, event) {
                ConfirmResult::None => {}
                ConfirmResult::Cancel => self.state.deny_default(),
                ConfirmResult::Ok => self.delete_save(*i),
            },
            Focus::ContextMenu(i) => match self.context_menu.key_event(backend, event) {
                SelectResult::Ok(j) => match self.context_menu.get(j) {
                    Some(CONTEXT_CANCEL) => self.state.cancel_default(),
                    Some(CONTEXT_DELETE) => self.state.select(Focus::DeletePopoup(*i)),
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
            Focus::Menu | Focus::Table => {
                if self.menu.mouse_event(backend) {
                    self.state.set(Focus::Menu);
                    true
                } else if self.table.mouse_event(backend) {
                    self.menu.state.select(None);
                    self.state.set(Focus::Table);
                    true
                } else {
                    false
                }
            }
            Focus::DeletePopoup(..) => self.delete_popup.mouse_event(backend),
            Focus::ContextMenu(..) => self.context_menu.mouse_event(backend),
        }
    }
}
