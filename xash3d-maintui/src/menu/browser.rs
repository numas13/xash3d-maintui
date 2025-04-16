use std::{
    fmt::Write,
    str,
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
use xash3d_shared::{parser::Tokens, raw::netadr_s};
use xash3d_ui::engine;

use crate::{
    input::{Key, KeyEvent},
    strings::strings,
    ui::{utils, Control, Menu, Screen, State},
    widgets::{InputPopup, InputResult, List, ListPopup, MyTable, SelectResult, WidgetMut},
};

use super::create_server::CreateServerMenu;

const DEFAULT_PORT: u16 = 27015;

const FAVORITE_SERVERS_PATH: &str = "favorite_servers.lst";
// const HISTORY_SERVERS_PATH: &str = "history_servers.lst";

const MENU_BACK: &str = "Back";
const MENU_CREATE_SERVER: &str = "#GameUI_GameMenu_CreateServer";
const MENU_ADD_FAVORITE: &str = "Add favorite server";
const MENU_REFRESH: &str = "Refresh";
const MENU_SORT: &str = "Sort";

const SORT_PING: &str = "Ping";
const SORT_NUMCL: &str = "#GameUI_CurrentPlayers";
const SORT_HOST: &str = "#GameUI_ServerName";
const SORT_MAP: &str = "#GameUI_Map";

const PROTOCOL_CANCEL: &str = "Cancel";
const PROTOCOL_XASH3D_49: &str = "Xash3D 49 (new)";
const PROTOCOL_XASH3D_48: &str = "Xash3D 48 (old)";
const PROTOCOL_GOLD_SOURCE_48: &str = "GoldSource 48";

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum SortBy {
    #[default]
    Ping,
    Numcl,
    Host,
    Map,
}

#[derive(Clone)]
struct ServerInfo {
    addr: netadr_s,
    host: String,
    host_cmp: String,
    map: String,
    gamedir: String,
    numcl: u32,
    maxcl: u32,
    legacy: bool,
    gs: bool,
    dm: bool,
    team: bool,
    coop: bool,
    password: bool,
    dedicated: bool,
    favorite: bool,
    fake: bool,
    protocol: u8,
    ping: Duration,
}

impl ServerInfo {
    // XXX: netadr_s does not implement Default trait...
    fn new(addr: netadr_s) -> Self {
        Self {
            addr,
            host: String::new(),
            host_cmp: String::new(),
            map: String::new(),
            gamedir: String::new(),
            numcl: 0,
            maxcl: 0,
            legacy: false,
            gs: false,
            dm: false,
            team: false,
            coop: false,
            password: false,
            dedicated: false,
            favorite: false,
            fake: false,
            protocol: 0,
            ping: Duration::default(),
        }
    }

    fn from(addr: netadr_s, info: &str, start: Instant) -> Option<Self> {
        use xash3d::color::trim_color;

        if !info.starts_with("\\") {
            return None;
        }

        let mut ret = Self::new(addr);
        ret.ping = start.elapsed();
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
        ret.host_cmp = xash3d_protocol::color::trim_color(&ret.host).to_lowercase();
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
        let engine = engine();
        let address = engine.addr_to_string(self.addr);
        trace!("Ui: connect to {address}");
        let mut cmd = CStrArray::<256>::new();
        write!(cmd.cursor(), "connect {address} {}", self.protocol()).unwrap();
        engine.set_cvar_string(c"password", password);
        engine.client_cmd(&cmd);
        Control::BackMain
    }
}

struct SavedServer {
    addr: netadr_s,
    // FIXME: replace with enum
    protocol: String,
}

impl SavedServer {
    fn new(addr: netadr_s, protocol: &str) -> Self {
        Self {
            addr,
            protocol: protocol.to_string(),
        }
    }

    fn query_info(&self) {
        let address = engine().addr_to_string(self.addr);
        engine().client_cmdf(format_args!(
            "queryserver \"{address}\" \"{}\"",
            self.protocol
        ));
    }

    fn fake_server_info(&self) -> ServerInfo {
        ServerInfo {
            host: engine().addr_to_string(self.addr).to_string(),
            legacy: self.protocol == "48",
            gs: self.protocol == "gs",
            protocol: match self.protocol.as_str() {
                "49" => 49,
                "48" | "legacy" => 48,
                "gs" | "goldsrc" => 48,
                _ => 0,
            },
            ping: Duration::from_secs(999),
            favorite: true,
            fake: true,
            ..ServerInfo::new(self.addr)
        }
    }
}

#[derive(Default)]
struct SavedServers {
    list: Vec<SavedServer>,
    changed: bool,
}

impl SavedServers {
    fn load_from_file(path: &str) -> Result<Self, &'static str> {
        let engine = engine();
        let file = engine.load_file(path).ok_or("failed to load")?;
        let data = str::from_utf8(file.as_slice()).map_err(|_| "invalid utf8")?;
        let mut tokens = Tokens::new(data).handle_colon(false);
        let mut servers = Self::default();
        while let Some((Ok(addr), Ok(protocol))) = tokens.next().zip(tokens.next()) {
            let Some(addr) = engine.string_to_addr(addr) else {
                warn!("invalid address {addr:?} in file \"{path}\"");
                continue;
            };
            if !servers.contains(&addr) {
                servers.list.push(SavedServer {
                    addr,
                    protocol: protocol.to_string(),
                });
            }
        }
        trace!("load {} servers from file \"{path}\"", servers.list.len());
        Ok(servers)
    }

    fn save_to_file(&self, path: &str) {
        if !self.changed {
            return;
        }
        let engine = engine();
        let mut out = String::new();
        let mut count = 0;
        for i in &self.list {
            count += 1;
            let address = engine.addr_to_string(i.addr);
            writeln!(&mut out, "{address} {}", i.protocol).unwrap();
        }
        if count > 0 {
            trace!("save {count} servers to file \"{path}\"");
            engine.save_file(path, out.as_bytes());
        } else {
            trace!("delete servers file \"{path}\"");
            engine.remove_file(path);
        }
    }

    fn insert(&mut self, addr: netadr_s, protocol: &str) -> Option<&SavedServer> {
        if !self.contains(&addr) {
            self.changed = true;
            self.list.push(SavedServer::new(addr, protocol));
            self.list.last()
        } else {
            None
        }
    }

    fn remove(&mut self, addr: &netadr_s) -> Option<SavedServer> {
        let engine = engine();
        self.list
            .iter()
            .position(|i| engine.compare_addr(&i.addr, addr))
            .map(|i| {
                self.changed = true;
                self.list.remove(i)
            })
    }

    fn contains(&self, addr: &netadr_s) -> bool {
        let engine = engine();
        self.list.iter().any(|i| engine.compare_addr(&i.addr, addr))
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
    AddFavoriteServer(Option<netadr_s>),
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    Direct,
    Favorite,
    Nat,
}

impl Tab {
    fn prev(&self) -> Tab {
        match self {
            Self::Direct => Self::Direct,
            Self::Favorite => Self::Direct,
            Self::Nat => Self::Favorite,
        }
    }

    fn next(&self) -> Tab {
        match self {
            Self::Direct => Self::Favorite,
            Self::Favorite => Self::Nat,
            Self::Nat => Self::Nat,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Direct => "Direct",
            Self::Favorite => "Favorite",
            Self::Nat => "NAT",
        }
    }
}

pub struct Browser {
    state: State<Focus>,
    is_lan: bool,
    sorted: bool,
    time: Instant,
    menu: List,
    sort_by: SortBy,
    sort_reverse: bool,
    sort_popup: ListPopup,
    password_popup: InputPopup,
    tab: Tab,
    table: MyTable<ServerInfo>,
    tabs: [(Tab, Rect); 3],
    favorite_servers: SavedServers,
    address_popup: InputPopup,
    protocol_popup: ListPopup,
}

impl Browser {
    pub fn new(is_lan: bool) -> Self {
        let items = if is_lan {
            &[MENU_BACK, MENU_CREATE_SERVER, MENU_SORT, MENU_REFRESH][..]
        } else {
            &[
                MENU_BACK,
                MENU_CREATE_SERVER,
                MENU_ADD_FAVORITE,
                MENU_SORT,
                MENU_REFRESH,
            ][..]
        };
        let mut menu = List::new_first(items);
        menu.state.select(None);
        menu.set_bindings([
            (Key::Char(b'a'), MENU_ADD_FAVORITE),
            (Key::Char(b'c'), MENU_CREATE_SERVER),
            (Key::Char(b'r'), MENU_REFRESH),
            (Key::Char(b'o'), MENU_SORT),
            (Key::Char(b'b'), MENU_BACK),
        ]);

        let mut favorite_servers = SavedServers::default();
        if !is_lan {
            match SavedServers::load_from_file(FAVORITE_SERVERS_PATH) {
                Ok(servers) => favorite_servers = servers,
                Err(err) => error!("{err}, file \"{FAVORITE_SERVERS_PATH}\""),
            }
        }

        Self {
            state: State::default(),
            is_lan,
            sorted: false,
            time: Instant::now(),
            menu,
            sort_by: SortBy::default(),
            sort_reverse: false,
            sort_popup: ListPopup::new(
                "Select sort column",
                [SORT_PING, SORT_NUMCL, SORT_HOST, SORT_MAP],
            ),
            password_popup: InputPopup::new_password("Password:"),
            tab: Tab::default(),
            table: MyTable::new_first(),
            tabs: [
                (Tab::Direct, Rect::ZERO),
                (Tab::Favorite, Rect::ZERO),
                (Tab::Nat, Rect::ZERO),
            ],
            favorite_servers,
            address_popup: InputPopup::new_text("Address:"),
            protocol_popup: ListPopup::new(
                "Select protocol",
                [
                    PROTOCOL_CANCEL,
                    PROTOCOL_XASH3D_49,
                    PROTOCOL_XASH3D_48,
                    PROTOCOL_GOLD_SOURCE_48,
                ],
            ),
        }
    }

    fn query_favorite_servers(&mut self) {
        for i in &self.favorite_servers.list {
            i.query_info();
            self.table.push(i.fake_server_info());
        }
        self.reset_ping();
    }

    fn query_servers(&mut self) {
        self.table.clear();
        let engine = engine();
        engine.set_cvar_float(c"cl_nat", if self.tab == Tab::Nat { 1.0 } else { 0.0 });
        match self.tab {
            Tab::Direct if self.is_lan => engine.client_cmd(c"localservers"),
            Tab::Direct => engine.client_cmd(c"internetservers"),
            Tab::Nat => engine.client_cmd(c"internetservers"),
            Tab::Favorite => self.query_favorite_servers(),
        }
    }

    fn switch_tab(&mut self, to: Tab) {
        if self.tab != to {
            self.tab = to;
            self.query_servers();
        }
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_CREATE_SERVER => {
                let public = if self.is_lan { 0.0 } else { 1.0 };
                engine().set_cvar_float(c"public", public);
                return Control::next(CreateServerMenu::new());
            }
            MENU_ADD_FAVORITE => {
                self.address_popup.clear();
                self.state.select(Focus::AddFavoriteServer(None));
            }
            MENU_REFRESH => self.query_servers(),
            MENU_SORT => self.show_sort_popup(matches!(self.state.focus(), Focus::Table)),
            MENU_BACK => return Control::Back,
            item => {
                warn!("{item} is not implemented yet")
            }
        }
        Control::None
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

    fn add_favorite(&mut self, addr: netadr_s, protocol: &str) -> bool {
        let engine = engine();
        if let Some(server) = self.favorite_servers.insert(addr, protocol) {
            let address = engine.addr_to_string_ref(&addr);
            trace!("add server \"{address}\" to favorite list");
            if let Some(server) = self
                .table
                .items
                .iter_mut()
                .find(|i| engine.compare_addr(&i.addr, &addr))
            {
                server.favorite = true;
            }
            if self.tab == Tab::Favorite {
                // FIXME: track ping for each server
                self.time = Instant::now();
                server.query_info();
                self.table.push(server.fake_server_info());
            }
            true
        } else {
            false
        }
    }

    fn protocol_popup_exec(&mut self, addr: netadr_s, i: usize) {
        let protocol = match &self.protocol_popup[i] {
            PROTOCOL_CANCEL => {
                self.state.cancel_default();
                return;
            }
            PROTOCOL_XASH3D_49 => "49",
            PROTOCOL_XASH3D_48 => "48",
            PROTOCOL_GOLD_SOURCE_48 => "gs",
            item => {
                warn!("{item} is not implemented yet");
                return;
            }
        };
        if self.add_favorite(addr, protocol) {
            self.state.confirm_default();
        } else {
            self.state.deny_default();
        }
    }

    fn sort_servers(&mut self) {
        self.table.sort_by(|a, b| {
            let o = match self.sort_by {
                SortBy::Ping => a.ping.cmp(&b.ping),
                SortBy::Numcl => a.numcl.cmp(&b.numcl).reverse(),
                SortBy::Host => a.host_cmp.cmp(&b.host_cmp),
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

    fn draw_tabs(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let areas = Layout::horizontal(self.tabs.iter().map(|_| Constraint::Fill(1))).split(area);
        for (i, area) in areas.iter().enumerate() {
            self.tabs[i].1 = *area;
        }
        for (tab, area) in self.tabs.iter() {
            let mut style = Style::default().white().on_dark_gray();
            if matches!(self.state.focus(), Focus::Tabs) {
                if *tab == self.tab {
                    style = style.black().on_yellow();
                } else if area.contains(screen.cursor) {
                    style = style.black().on_green();
                }
            } else if *tab == self.tab {
                style = style.black().on_green();
            }
            Line::raw(tab.as_str())
                .style(style)
                .centered()
                .render(*area, buf);
        }
    }

    fn draw_table(&mut self, area: Rect, buf: &mut Buffer) {
        let widths = [
            Constraint::Length(1),
            Constraint::Min(30),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Max(7),
        ];
        let sort_hint = |s, e: SortBy| {
            if self.sort_by == e {
                let p = if self.sort_reverse { "↑" } else { "↓" };
                Cell::new(Line::from_iter([p, s]))
            } else {
                Cell::new(s)
            }
        };
        let strings = strings();
        let header = Row::new([
            Cell::new(""),
            sort_hint(strings.get("#GameUI_ServerName"), SortBy::Host),
            sort_hint(strings.get("#GameUI_Map"), SortBy::Map),
            Cell::new(""),
            sort_hint("Players", SortBy::Numcl),
            sort_hint("Ping", SortBy::Ping),
        ]);
        let table = self
            .table
            .create_table(area, header.style(Style::new().on_black()), &widths);

        let focused = matches!(self.state.focus(), Focus::Table);
        self.table.draw(area, buf, table, focused, |i| {
            let cells = [
                Cell::new(if i.favorite { "*" } else { "" }),
                Cell::new(colorize(i.host.as_str())),
                Cell::new(i.map.as_str()),
                Cell::new(if i.password { "[P]" } else { "" }),
                Cell::new(Span::from(format!("{}/{}", i.numcl, i.maxcl)).into_centered_line()),
                Cell::new(format!("{:.0?}", i.ping)),
            ];
            let row = Row::new(cells);
            if i.fake {
                Some(row.style(Style::new().dark_gray()))
            } else {
                Some(row)
            }
        });
    }

    fn handle_mouse_click(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let cursor = backend.cursor_position();
        if let Some((t, _)) = self.tabs.iter().find(|(_, i)| i.contains(cursor)) {
            self.tab = *t;
            self.query_servers();
        } else if self.menu.area.contains(cursor) {
            return self.menu_key_event(backend, event);
        } else if let Some(column) = self.table.cursor_to_header_column(cursor) {
            match column {
                0 => {} // favorite
                1 => self.set_sort(SortBy::Host),
                2 => self.set_sort(SortBy::Map),
                3 => {} // password
                4 => self.set_sort(SortBy::Numcl),
                5 => self.set_sort(SortBy::Ping),
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
            Key::Tab => {
                self.tabs_key_event(backend, event);
            }
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
        if self.is_lan {
            return Control::None;
        }
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
            Key::Tab if event.shift() => self.switch_tab(self.tab.prev()),
            Key::Tab => self.switch_tab(self.tab.next()),
            Key::Char(b'h') | Key::ArrowLeft => self.switch_tab(self.tab.prev()),
            Key::Char(b'l') | Key::ArrowRight => self.switch_tab(self.tab.next()),
            Key::Char(b'q') => return Control::Back,
            Key::Mouse(0) => return self.handle_mouse_click(backend, event),
            _ => {}
        }
        Control::None
    }

    fn toggle_favorite(&mut self) {
        if self.is_lan || matches!(self.tab, Tab::Nat) {
            return;
        }
        let Some(selected) = self.table.state.selected() else {
            return;
        };
        if let Some(server) = self.table.items.get_mut(selected) {
            let address = engine().addr_to_string_ref(&server.addr);
            if self.favorite_servers.remove(&server.addr).is_some() {
                server.favorite = false;
                trace!("remove server \"{address}\" from favorite list");
            } else {
                server.favorite = true;
                trace!("add server \"{address}\" to favorite list");
                self.favorite_servers.insert(server.addr, server.protocol());
            }
        }
    }

    fn table_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match key {
            Key::Char(b'h') | Key::ArrowLeft if self.tab == Tab::Direct => return Control::Back,
            Key::Tab | Key::Char(b'h' | b'l') | Key::ArrowLeft | Key::ArrowRight => {
                self.tabs_key_event(backend, event);
            }
            Key::Char(b'f') => self.toggle_favorite(),
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

impl Drop for Browser {
    fn drop(&mut self) {
        if !self.is_lan {
            self.favorite_servers.save_to_file(FAVORITE_SERVERS_PATH);
        }
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
            self.draw_tabs(tabs_area, buf, screen);
            self.draw_table(table_area, buf);
        } else {
            self.draw_table(table_area, buf);
        }

        match self.state.focus() {
            Focus::SortPopup(_) => self.sort_popup.render(area, buf, screen),
            Focus::PasswordPopup(_) => self.password_popup.render(area, buf, screen),
            Focus::AddFavoriteServer(None) => self.address_popup.render(area, buf, screen),
            Focus::AddFavoriteServer(Some(_)) => self.protocol_popup.render(area, buf, screen),
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
            Focus::Menu => return self.menu_key_event(backend, event),
            Focus::Tabs => return self.tabs_key_event(backend, event),
            Focus::Table => return self.table_key_event(backend, event),
            Focus::AddFavoriteServer(None) => match self.address_popup.key_event(backend, event) {
                InputResult::Ok(address) => match engine().string_to_addr(address.as_str()) {
                    Some(mut addr) => {
                        if addr.port == 0 {
                            addr.port = DEFAULT_PORT.to_be();
                        }
                        self.state.confirm(Focus::AddFavoriteServer(Some(addr)));
                        self.protocol_popup.state.select(Some(1));
                    }
                    None => {
                        // TODO: print error message
                        error!("invalid server address {address:?}");
                        self.state.deny_default();
                    }
                },
                InputResult::Cancel => self.state.cancel_default(),
                _ => {}
            },
            Focus::AddFavoriteServer(Some(address)) => {
                match self.protocol_popup.key_event(backend, event) {
                    SelectResult::Ok(i) => self.protocol_popup_exec(*address, i),
                    SelectResult::Cancel => self.state.cancel_default(),
                    _ => {}
                }
            }
            Focus::SortPopup(focus_table) => match self.sort_popup.key_event(backend, event) {
                SelectResult::Cancel => self.state.select(Focus::Table),
                SelectResult::Ok(i) => self.sort_item_exec(i, *focus_table),
                _ => {}
            },
            Focus::PasswordPopup(server) => match self.password_popup.key_event(backend, event) {
                InputResult::Ok(password) => return server.connect(&password),
                InputResult::Cancel => {
                    self.state.cancel_default();
                    return Control::GrabInput(false);
                }
                _ => {}
            },
        }
        Control::None
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        match self.state.focus() {
            Focus::SortPopup(_) => self.sort_popup.mouse_event(backend),
            Focus::PasswordPopup(_) => self.password_popup.mouse_event(backend),
            Focus::AddFavoriteServer(None) => self.address_popup.mouse_event(backend),
            Focus::AddFavoriteServer(Some(_)) => self.protocol_popup.mouse_event(backend),
            _ => {
                if self.menu.mouse_event(backend) {
                    self.state.set(Focus::Menu);
                    true
                // TODO: highlight table header
                } else if self.table.mouse_event(backend) {
                    self.menu.state.select(None);
                    self.state.set(Focus::Table);
                    true
                } else if self
                    .tabs
                    .iter()
                    .any(|i| i.1.contains(backend.cursor_position()))
                {
                    self.menu.state.select(None);
                    self.state.set(Focus::Tabs);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn add_server_to_list(&mut self, addr: netadr_s, info: &str) {
        let engine = engine();
        match ServerInfo::from(addr, info, self.time) {
            Some(mut info) => {
                info.fake = false;
                if let Some(i) = self
                    .table
                    .iter_mut()
                    .find(|i| engine.compare_addr(&i.addr, &addr))
                {
                    *i = ServerInfo {
                        favorite: i.favorite,
                        ..info
                    };
                    self.sorted = false;
                } else {
                    if !self.is_lan && self.tab != Tab::Nat {
                        info.favorite = self.favorite_servers.contains(&addr);
                    }
                    if self.tab != Tab::Favorite || info.favorite {
                        self.table.push(info);
                        self.sorted = false;
                    }
                }
                if matches!(self.state.focus(), Focus::Table) && self.table.len() == 1 {
                    self.table.state.select_first();
                }
            }
            None => {
                let address = engine.addr_to_string(addr);
                trace!("failed to add server {address} with info {info}");
            }
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
