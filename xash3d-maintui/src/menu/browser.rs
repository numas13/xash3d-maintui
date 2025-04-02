use std::{
    fmt::Write,
    time::{Duration, Instant},
};

use csz::CStrArray;
use ratatui::{
    prelude::*,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Cell, Row},
};
use xash3d_protocol::{self as xash3d, color::Color as XashColor};
use xash3d_ratatui::XashBackend;
use xash3d_shared::raw::netadr_s;
use xash3d_ui::engine;

use crate::{
    input::{Key, KeyEvent},
    strings::strings,
    ui::{utils, Control, Menu, Screen, State},
    widgets::{
        InputResult, List, ListPopup, MyTable, PasswordPopup, SelectResult, TableHeader, WidgetMut,
    },
};

use super::create_server::CreateServerMenu;

const MENU_BACK: &str = "Back";
const MENU_CREATE_SERVER: &str = "#GameUI_GameMenu_CreateServer";
const MENU_REFRESH: &str = "Refresh";
const MENU_SORT: &str = "Sort";

const SORT_PING: &str = "Ping";
const SORT_NUMCL: &str = "#GameUI_CurrentPlayers";
const SORT_HOST: &str = "#GameUI_ServerName";
const SORT_MAP: &str = "#GameUI_Map";

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum SortBy {
    #[default]
    Ping,
    Numcl,
    Host,
    Map,
}

#[derive(Clone, Default)]
struct ServerInfo {
    address: String,
    host: String,
    map: String,
    gamedir: String,
    numcl: u32,
    maxcl: u32,
    protocol: u8,
    legacy: bool,
    gs: bool,
    dm: bool,
    team: bool,
    coop: bool,
    password: bool,
    dedicated: bool,
    ping: Duration,
}

impl ServerInfo {
    fn from(address: &str, info: &str, start: Instant) -> Option<Self> {
        use xash3d::color::trim_color;

        if !info.starts_with("\\") {
            return None;
        }

        let mut ret = Self {
            ping: start.elapsed(),
            ..Self::default()
        };

        let mut it = info[1..].split('\\');
        while let Some(key) = it.next() {
            let value = it.next()?;
            match key {
                "p" => ret.protocol = trim_color(value).parse().unwrap_or_default(),
                "host" => ret.host = value.trim().to_owned(),
                "map" => ret.map = trim_color(value).to_string(),
                "gamedir" => ret.gamedir = trim_color(value).to_string(),
                "numcl" => ret.numcl = trim_color(value).parse().unwrap_or_default(),
                "maxcl" => ret.maxcl = trim_color(value).parse().unwrap_or_default(),
                "legacy" => {
                    ret.ping /= 2;
                    ret.legacy = value == "1";
                }
                "gs" => ret.gs = value == "1",
                "dm" => ret.dm = value == "1",
                "team" => ret.team = value == "1",
                "coop" => ret.coop = value == "1",
                "password" => ret.password = value == "1",
                "dedicated" => ret.dedicated = value == "1",
                _ => debug!("unimplemented server info {key}={value}"),
            }
        }
        ret.address = address.to_owned();
        Some(ret)
    }

    fn protocol(&self) -> &str {
        if self.legacy {
            "48"
        } else if self.gs {
            "gs"
        } else {
            "49"
        }
    }

    fn connect(&self, password: &str) -> Control {
        trace!("Ui: connect to {}", self.address);
        let mut cmd = CStrArray::<256>::new();
        write!(cmd.cursor(), "connect {} {}", self.address, self.protocol()).unwrap();
        let engine = engine();
        engine.set_cvar_string(c"password", password);
        engine.client_cmd(&cmd);
        Control::BackMain
    }
}

#[derive(Clone, Default)]
enum Focus {
    Menu,
    Tabs,
    #[default]
    Table,
    SortPopup(bool),
    PasswordPopup(ServerInfo),
}

pub struct Browser {
    state: State<Focus>,
    is_lan: bool,
    nat: bool,
    sorted: bool,
    time: Instant,
    menu: List,
    sort_by: SortBy,
    sort_reverse: bool,
    sort_popup: ListPopup,
    password_popup: PasswordPopup,
    table_header: TableHeader,
    table: MyTable<ServerInfo>,
    tabs_area: [Rect; 2],
}

impl Browser {
    pub fn new(is_lan: bool) -> Self {
        let strings = strings();
        let mut menu = List::new_first([MENU_BACK, MENU_CREATE_SERVER, MENU_SORT, MENU_REFRESH]);
        menu.state.select(None);
        menu.set_bindings([
            (Key::Char(b'c'), MENU_CREATE_SERVER),
            (Key::Char(b'r'), MENU_REFRESH),
            (Key::Char(b'o'), MENU_SORT),
            (Key::Char(b'b'), MENU_BACK),
        ]);

        let table_header = TableHeader::new([
            strings.get("#GameUI_ServerName"),
            strings.get("#GameUI_Map"),
            "",
            "Players",
            "Ping",
        ]);

        Self {
            state: State::default(),
            is_lan,
            nat: false,
            sorted: false,
            time: Instant::now(),
            menu,
            sort_by: SortBy::default(),
            sort_reverse: false,
            sort_popup: ListPopup::new(
                "Select sort column",
                [SORT_PING, SORT_NUMCL, SORT_HOST, SORT_MAP],
            ),
            password_popup: PasswordPopup::new("Password:"),
            table_header,
            table: MyTable::new_first(),
            tabs_area: [Rect::ZERO; 2],
        }
    }

    fn query_servers(&mut self) {
        self.table.clear();
        let engine = engine();
        if self.is_lan {
            engine.client_cmd(c"localservers");
        } else {
            let nat = if self.nat { 1.0 } else { 0.0 };
            engine.set_cvar_float(c"cl_nat", nat);
            engine.client_cmd(c"internetservers");
        }
    }

    fn switch_tab(&mut self, nat: bool) {
        self.nat = nat;
        self.query_servers();
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_CREATE_SERVER => {
                let public = if self.is_lan { 0.0 } else { 1.0 };
                engine().set_cvar_float(c"public", public);
                Control::next(CreateServerMenu::new())
            }
            MENU_REFRESH => {
                self.query_servers();
                Control::None
            }
            MENU_SORT => {
                self.show_sort_popup(matches!(self.state.focus(), Focus::Table));
                Control::None
            }
            MENU_BACK => Control::Back,
            item => {
                warn!("{item} is not implemented yet");
                Control::None
            }
        }
    }

    fn table_exec(&mut self, i: usize) -> Control {
        let Some(server) = self.table.get(i) else {
            return Control::None;
        };
        if server.password {
            self.state.select(Focus::PasswordPopup(server.clone()));
            return Control::GrabInput(true);
        }
        server.connect("")
    }

    fn sort_servers(&mut self) {
        self.table.sort_by(|a, b| {
            let o = match self.sort_by {
                SortBy::Ping => a.ping.cmp(&b.ping),
                SortBy::Numcl => a.numcl.cmp(&b.numcl).reverse(),
                SortBy::Host => a.host.cmp(&b.host),
                SortBy::Map => a.map.cmp(&b.map),
            };
            match self.sort_reverse {
                false => o,
                true => o.reverse(),
            }
        });
    }

    fn show_sort_popup(&mut self, focus_table: bool) {
        self.state.select(Focus::SortPopup(focus_table));
    }

    fn set_sort(&mut self, sort_by: SortBy) {
        if self.sort_by == sort_by {
            self.sort_reverse = !self.sort_reverse;
        } else {
            self.sort_reverse = false;
            self.sort_by = sort_by;
        }
        self.sorted = false;
    }

    fn sort_item_exec(&mut self, i: usize, focus_table: bool) {
        let sort_by = match &self.sort_popup[i] {
            SORT_PING => SortBy::Ping,
            SORT_NUMCL => SortBy::Numcl,
            SORT_HOST => SortBy::Host,
            SORT_MAP => SortBy::Map,
            _ => {
                debug!("unimplemented sort popup item {i}");
                return;
            }
        };
        self.set_sort(sort_by);
        if focus_table {
            self.table.state.select_first();
            self.state.select(Focus::Table);
        } else {
            self.state.select(Focus::Menu);
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

    fn draw_tabs(&mut self, area: Rect, buf: &mut Buffer) {
        self.tabs_area =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        for (text, area) in ["Direct", "NAT"].into_iter().zip(self.tabs_area) {
            let mut style = Style::default().black().on_dark_gray();
            if (text == "NAT") == self.nat {
                if matches!(self.state.focus(), Focus::Tabs) {
                    style = style.on_yellow();
                } else {
                    style = style.on_green();
                }
            } else {
                style = style.white();
            }
            Line::raw(text).style(style).centered().render(area, buf);
        }
    }

    fn draw_table(&mut self, area: Rect, buf: &mut Buffer) {
        let widths = [
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Max(7),
        ];
        // TODO: hint sorted column
        let table = self
            .table_header
            .create_table(area, &widths, Style::new().on_dark_gray());

        let focused = matches!(self.state.focus(), Focus::Table);
        self.table.draw(area, buf, table, focused, |i| {
            let cells = [
                Cell::new(colorize(i.host.as_str())),
                Cell::new(i.map.as_str()),
                Cell::new(if i.password { "[P]" } else { "" }),
                Cell::new(Span::from(format!("{}/{}", i.numcl, i.maxcl)).into_centered_line()),
                Cell::new(format!("{:.0?}", i.ping)),
            ];
            Some(Row::new(cells))
        });
    }

    fn handle_mouse_click(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let cursor = backend.cursor_position();
        let index = self.tabs_area.iter().position(|i| i.contains(cursor));
        if let Some(i) = index {
            self.nat = i == 1;
            self.query_servers();
        } else if self.menu.area.contains(cursor) {
            return self.menu_key_event(backend, event);
        } else if let Some(column) = self.table_header.contains(cursor) {
            match column {
                0 => self.set_sort(SortBy::Host),
                1 => self.set_sort(SortBy::Map),
                2 => {} // password
                3 => self.set_sort(SortBy::Numcl),
                4 => self.set_sort(SortBy::Ping),
                _ => debug!("unimplemented click to table header {column}"),
            }
        } else if self.table.area.contains(cursor) {
            return self.table_key_event(backend, event);
        }
        Control::None
    }

    fn menu_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match key {
            Key::Tab => self.switch_tab(!self.nat),
            _ => match self.menu.key_event(backend, event) {
                SelectResult::Ok(i) => return self.menu_exec(i),
                SelectResult::Down if self.is_lan => {
                    self.menu.state.select(None);
                    self.table.state.select_first();
                    self.state.next(Focus::Table);
                }
                SelectResult::Down => {
                    self.menu.state.select(None);
                    self.state.next(Focus::Tabs);
                }
                SelectResult::Cancel => return Control::Back,
                _ => {}
            },
        }
        Control::None
    }

    fn tabs_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match key {
            _ if key.is_prev() => {
                self.menu.state.select_last();
                self.state.prev(Focus::Menu);
            }
            _ if key.is_next() => {
                self.table.state.select_first();
                self.state.next(Focus::Table);
            }
            Key::Tab => self.switch_tab(!self.nat),
            Key::Char(b'h') | Key::ArrowLeft => self.switch_tab(false),
            Key::Char(b'l') | Key::ArrowRight => self.switch_tab(true),
            Key::Char(b'q') => return Control::Back,
            Key::Mouse(0) => return self.handle_mouse_click(backend, event),
            _ => {}
        }
        Control::None
    }

    fn table_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match key {
            Key::Tab => self.switch_tab(!self.nat),
            _ => match self.table.key_event(backend, event) {
                SelectResult::Ok(i) => return self.table_exec(i),
                SelectResult::Up if self.is_lan => {
                    self.menu.state.select_last();
                    self.table.state.select(None);
                    self.state.prev(Focus::Menu);
                }
                SelectResult::Up => {
                    self.table.state.select(None);
                    self.state.prev(Focus::Tabs);
                }
                SelectResult::Cancel => return Control::Back,
                _ => {}
            },
        }
        Control::None
    }
}

impl Menu for Browser {
    fn active(&mut self) {
        self.query_servers();
    }

    fn on_menu_hide(&mut self) {
        self.state.reset();
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        if !self.sorted {
            self.sort_servers();
            self.sorted = true;
        }

        let title = if self.is_lan {
            "Local servers"
        } else {
            "Internet servers"
        };
        let [menu_area, table_area] = Layout::vertical([
            Constraint::Length(self.menu.len() as u16 + 1),
            Constraint::Percentage(100),
        ])
        .areas(utils::main_block(title, area, buf));

        self.draw_menu(menu_area, buf, screen);
        if !self.is_lan {
            let [tabs_area, table_area] =
                Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)])
                    .areas(table_area);
            self.draw_tabs(tabs_area, buf);
            self.draw_table(table_area, buf);
        } else {
            self.draw_table(table_area, buf);
        }

        match self.state.focus() {
            Focus::SortPopup(_) => self.sort_popup.render(area, buf, screen),
            Focus::PasswordPopup(_) => self.password_popup.render(area, buf, screen),
            _ => {}
        }
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        if matches!(self.state.focus(), Focus::Menu | Focus::Tabs | Focus::Table) {
            if let Some(i) = self.menu.match_binding(event.key()) {
                return self.menu_exec(i);
            }
            if let Key::Mouse(0) = event.key() {
                return self.handle_mouse_click(backend, event);
            }
        }
        match self.state.focus() {
            Focus::Menu => self.menu_key_event(backend, event),
            Focus::Tabs => self.tabs_key_event(backend, event),
            Focus::Table => self.table_key_event(backend, event),
            Focus::SortPopup(focus_table) => {
                match self.sort_popup.key_event(backend, event) {
                    SelectResult::Cancel => self.state.select(Focus::Table),
                    SelectResult::Ok(i) => self.sort_item_exec(i, *focus_table),
                    _ => {}
                }
                Control::None
            }
            Focus::PasswordPopup(server) => match self.password_popup.key_event(backend, event) {
                InputResult::Ok(password) => server.connect(&password),
                InputResult::Cancel => {
                    self.state.cancel_default();
                    Control::GrabInput(false)
                }
                _ => Control::None,
            },
        }
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        if let Focus::SortPopup(_) = self.state.focus() {
            self.sort_popup.mouse_event(backend)
        } else if let Focus::PasswordPopup(_) = self.state.focus() {
            self.password_popup.mouse_event(backend)
        } else if self.menu.mouse_event(backend) {
            self.state.set(Focus::Menu);
            true
        // TODO: highlight table header
        } else if self.table.mouse_event(backend) {
            self.menu.state.select(None);
            self.state.set(Focus::Table);
            true
        } else if self
            .tabs_area
            .iter()
            .any(|i| i.contains(backend.cursor_position()))
        {
            self.menu.state.select(None);
            self.state.set(Focus::Tabs);
            true
        } else {
            false
        }
    }

    fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {
        let addr = engine().addr_to_string(addr);
        let Ok(addr) = addr.to_str() else {
            error!("invalid server address: {addr}");
            return;
        };
        match ServerInfo::from(addr, info, self.time) {
            Some(info) => {
                if self.table.iter().any(|i| i.address == addr) {
                    return;
                }
                self.table.push(info);
                self.sorted = false;
                if matches!(self.state.focus(), Focus::Table) && self.table.len() == 1 {
                    self.table.state.select_first();
                }
            }
            None => trace!("failed to add server {addr} with info {info}"),
        }
    }

    fn reset_ping(&mut self) {
        self.time = Instant::now();
    }
}

fn colorize(s: &str) -> Line {
    let mut line = Line::default();
    for (color, text) in xash3d::color::ColorIter::new(s) {
        let style = XashColor::try_from(color)
            .map(|color| {
                let color = match color {
                    XashColor::Black => Color::Black,
                    XashColor::Red => Color::Red,
                    XashColor::Green => Color::Green,
                    XashColor::Yellow => Color::Yellow,
                    XashColor::Blue => Color::Blue,
                    XashColor::Cyan => Color::Cyan,
                    XashColor::Magenta => Color::Magenta,
                    XashColor::White => Color::White,
                };
                Style::new().fg(color)
            })
            .unwrap_or_default();
        line.push_span(Span::from(text).style(style))
    }
    line
}
