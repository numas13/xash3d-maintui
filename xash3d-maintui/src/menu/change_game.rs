use compact_str::{CompactString, ToCompactString};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Row, Table},
};
use xash3d_ratatui::XashBackend;
use xash3d_ui::game_info::GameInfo2;

use crate::{
    input::KeyEvent,
    prelude::*,
    strings::Localize,
    ui::{utils, Control, Menu, Screen, State},
    widgets::{ConfirmPopup, ConfirmResult, List, MyTable, SelectResult, WidgetMut},
};

mod i18n {
    pub use crate::i18n::{all::*, menu::custom_game::*};
}

const MENU_BACK: &str = i18n::BACK;

struct GameInfo {
    active: bool,
    ty: CompactString,
    gamedir: CompactString,
    name: CompactString,
    version: CompactString,
    size: CompactString,
}

impl From<&GameInfo2> for GameInfo {
    fn from(info: &GameInfo2) -> Self {
        Self {
            active: false,
            ty: info.game_type().to_compact_string(),
            gamedir: info.game_dir().to_compact_string(),
            name: info.title().to_compact_string(),
            version: info.game_version().to_compact_string(),
            size: utils::pretty_size(info.size()).to_compact_string(),
        }
    }
}

#[derive(Copy, Clone, Default)]
enum Focus {
    #[default]
    Menu,
    Table,
    ConfirmPopup(usize),
}

pub struct ChangeGame {
    state: State<Focus>,
    menu: List,
    table: MyTable<GameInfo>,
    change_popup: ConfirmPopup,
}

impl ChangeGame {
    pub fn new() -> Self {
        let mut table = MyTable::default();

        let engine = engine();
        let gamedir = engine.get_game_dir();
        for i in engine.mod_info_iter() {
            table.push(GameInfo {
                active: i.game_dir() == gamedir.as_thin(),
                ..GameInfo::from(i)
            });
        }

        Self {
            state: State::default(),
            menu: List::new_first([MENU_BACK]),
            table,
            change_popup: ConfirmPopup::with_title(
                i18n::CHANGE_POPUP_TITLE,
                i18n::CHANGE_POPUP_BODY,
            ),
        }
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_BACK => return Control::Back,
            item => warn!("{item} is not implemented yet"),
        }
        Control::None
    }

    fn table_exec(&mut self, i: usize) -> Control {
        if !self.table[i].active {
            self.state.select(Focus::ConfirmPopup(i));
        } else {
            trace!("game already active");
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
            "",
            i18n::COLUMN_TYPE.localize(),
            i18n::COLUMN_NAME.localize(),
            i18n::COLUMN_VERSION.localize(),
            i18n::COLUMN_SIZE.localize(),
        ]);
        let table = Table::default()
            .header(header.style(Style::new().on_black()))
            .widths([
                Constraint::Length(1),
                Constraint::Length(12),
                Constraint::Min(20),
                Constraint::Length(8),
                Constraint::Length(10),
            ]);

        let focused = !matches!(self.state.focus(), Focus::Menu);
        self.table.draw(area, buf, table, focused, |i| {
            let cells = [
                if i.active { "*" } else { "" },
                i.ty.as_str(),
                i.name.as_str(),
                i.version.as_str(),
                i.size.as_str(),
            ];
            let row = Row::new(cells);
            if i.active {
                Some(row.style(Style::default().on_dark_gray()))
            } else {
                Some(row)
            }
        });
    }
}

impl Menu for ChangeGame {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let inner_area = utils::main_block(i18n::TITLE, area, buf);
        let [menu_area, table_area] = Layout::vertical([
            Constraint::Length(self.menu.len() as u16 + 1),
            Constraint::Percentage(100),
        ])
        .areas(inner_area);

        self.draw_menu(menu_area, buf, screen);
        self.draw_table(table_area, buf);

        if matches!(self.state.focus(), Focus::ConfirmPopup(..)) {
            self.change_popup.render(area, buf, screen);
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
                match self.table.key_event(backend, event) {
                    SelectResult::Ok(i) => return self.table_exec(i),
                    SelectResult::Up => {
                        self.table.state.select(None);
                        self.menu.state.select_last();
                        self.state.prev(Focus::Menu);
                    }
                    SelectResult::Cancel => return Control::Back,
                    _ => {}
                }
            }
            Focus::ConfirmPopup(i) => match self.change_popup.key_event(backend, event) {
                ConfirmResult::Ok => {
                    engine().client_cmd(format_args!("game {}\n", self.table[*i].gamedir));
                }
                ConfirmResult::Cancel => self.state.cancel(Focus::Table),
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
            Focus::ConfirmPopup(..) => self.change_popup.mouse_event(backend),
        }
    }
}
