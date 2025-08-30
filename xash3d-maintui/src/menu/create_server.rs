use core::{
    cell::RefCell,
    ffi::CStr,
    fmt::{self, Write},
    str,
};

use alloc::{rc::Rc, string::String, vec::Vec};
use compact_str::{CompactString, ToCompactString};
use csz::CStrArray;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{
    engine,
    parser::{Error as ParseError, Tokens},
    utils::escape_command,
};

use crate::{
    config_list::{Button, ConfigBackend, ConfigEntry, ConfigItem, ConfigList},
    input::KeyEvent,
    strings::Localize,
    ui::{Control, Menu, Screen},
    widgets::{Checkbox, Input, ListPopup},
};

mod i18n {
    pub use crate::i18n::menu::create_server::*;
}

const CVAR_HOST_SERVERSTATE: &CStr = c"host_serverstate";
const CVAR_HOSTNAME: &CStr = c"hostname";
const CVAR_SV_PASSWORD: &CStr = c"sv_password";
const CVAR_MAXPLAYERS: &CStr = c"maxplayers";
const CVAR_PUBLIC: &CStr = c"public";
const CVAR_SV_NAT: &CStr = c"sv_nat";

struct Map {
    name: CompactString,
    #[allow(dead_code)]
    title: CompactString,
}

fn parse_map_list(s: &str) -> Result<Vec<Map>, ParseError<'_>> {
    let mut list = Vec::new();
    list.push(Map {
        name: i18n::RANDOM_MAP.localize().into(),
        title: i18n::RANDOM_MAP_TITLE.localize().into(),
    });
    let mut tokens = Tokens::new(s);
    while let Some(name) = tokens.next() {
        let Some(title) = tokens.next() else {
            error!("unexpected end of file in maps.lst");
            break;
        };
        list.push(Map {
            name: name?.into(),
            title: title?.into(),
        })
    }
    Ok(list)
}

fn get_map_list() -> Option<Vec<Map>> {
    let engine = engine();
    if !engine.create_maps_list(true) {
        return None;
    }
    let Some(file) = engine.load_file(c"maps.lst") else {
        error!("failed to load maps.lst");
        return None;
    };
    // XXX: lossy because maps.lst can be encoded in any combination of encodings.
    let content = String::from_utf8_lossy(file.as_slice());
    match parse_map_list(&content) {
        Ok(list) => Some(list),
        Err(e) => {
            error!("failed to parse maps.lst: {e:?}");
            None
        }
    }
}

struct ServerParameters {
    server_name: CompactString,
    map_index: usize,
    password: CompactString,
    max_players: u32,
    nat: bool,
}

impl Default for ServerParameters {
    fn default() -> Self {
        let engine = engine();
        let mut server_name = engine.get_cvar_string(CVAR_HOSTNAME);
        if server_name.is_empty() {
            server_name = c"Xash3D Server".into();
        }
        let password = engine.get_cvar_string(CVAR_SV_PASSWORD);
        Self {
            server_name: server_name.to_compact_string(),
            map_index: 0,
            password: password.to_compact_string(),
            max_players: 16,
            nat: engine.get_cvar_float(CVAR_SV_NAT) != 0.0,
        }
    }
}

#[derive(Default)]
struct CreateServer {
    parms: RefCell<ServerParameters>,
    maps: Vec<Map>,
}

impl CreateServer {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            parms: Default::default(),
            maps: get_map_list().unwrap_or_default(),
        })
    }

    fn create_command(&self, buf: &mut CStrArray<256>, max_players: u32, map: &str) -> fmt::Result {
        let cur = &mut buf.cursor();
        write!(cur, "disconnect;")?;
        // TODO:
        // write!(cur, "menu_connectionprogress localserver;")?;
        write!(cur, "wait;wait;wait;")?;
        writeln!(cur, "maxplayers {max_players};")?;
        // TODO:
        // writeln!(cur, "latch;")?;
        writeln!(cur, "map {}", escape_command(map))
    }

    fn save_cvars(&self, parms: &ServerParameters) {
        let engine = engine();
        engine.set_cvar_string(CVAR_HOSTNAME, parms.server_name.as_str());
        engine.set_cvar_string(CVAR_SV_PASSWORD, parms.password.as_str());
        engine.set_cvar_float(CVAR_MAXPLAYERS, parms.max_players as f32);
        let nat = engine.get_cvar_float(CVAR_PUBLIC) != 0.0 && parms.nat;
        warn!("nat {nat}");
        engine.cvar_set(CVAR_SV_NAT, nat);
    }

    fn start(&self) {
        let parms = self.parms.borrow();
        let engine = engine();
        let mut buf = CStrArray::<256>::new();

        let map_index = if parms.map_index == 0 {
            engine.random_int(1, self.maps.len() as _) as usize
        } else {
            parms.map_index
        };
        let map = self.maps[map_index].name.as_str();

        if engine.get_cvar_float(CVAR_HOST_SERVERSTATE) != 0.0 {
            let s = if engine.get_cvar_float(CVAR_MAXPLAYERS) == 1.0 {
                c"end of the game"
            } else {
                c"starting new server"
            };
            engine.host_end_game(s);
        }

        // start deathmatch as default
        engine.set_cvar_float(c"deathmatch", 1.0);
        self.save_cvars(&parms);
        engine.stop_background_track();

        let listenservercfg = engine.get_cvar_string(c"lservercfgfile");
        engine.write_server_config(listenservercfg);
        engine.client_cmd_now(format_args!("exec {listenservercfg}\n"));

        // dirty listenserver config form old xash may rewrite maxplayers
        engine.set_cvar_float(CVAR_MAXPLAYERS, parms.max_players as f32);

        self.create_command(&mut buf, parms.max_players, map)
            .unwrap();
        engine.client_cmd(buf.as_thin());
    }

    fn start_button(self: &Rc<Self>) -> impl ConfigItem {
        let create_server = self.clone();
        Button::new(i18n::START_BUTTON.localize(), move || {
            create_server.start();
            Control::BackMainHide
        })
    }

    fn server_name_field(self: &Rc<Self>) -> impl ConfigItem {
        struct ServerName(Rc<CreateServer>);
        impl ConfigBackend<CompactString> for ServerName {
            fn read(&self) -> Option<CompactString> {
                Some(self.0.parms.borrow().server_name.clone())
            }

            fn write(&mut self, value: CompactString) {
                self.0.parms.borrow_mut().server_name = value;
            }
        }
        ConfigEntry::builder(Input::new())
            .label(i18n::NAME_LABEL.localize())
            .build(ServerName(self.clone()))
    }

    fn password_field(self: &Rc<Self>) -> impl ConfigItem {
        struct Password(Rc<CreateServer>);
        impl ConfigBackend<CompactString> for Password {
            fn read(&self) -> Option<CompactString> {
                Some(self.0.parms.borrow().password.clone())
            }

            fn write(&mut self, value: CompactString) {
                self.0.parms.borrow_mut().password = value;
            }
        }
        let widget = Input::builder().password().build();
        ConfigEntry::builder(widget)
            .label(i18n::PASSWORD_LABEL.localize())
            .build(Password(self.clone()))
    }

    fn map_list(self: &Rc<Self>) -> impl ConfigItem {
        struct MapList(Rc<CreateServer>);
        impl ConfigBackend<usize> for MapList {
            fn read(&self) -> Option<usize> {
                Some(self.0.parms.borrow().map_index)
            }

            fn write(&mut self, value: usize) {
                self.0.parms.borrow_mut().map_index = value;
            }
        }
        let title = i18n::MAPS_TITLE.localize();
        let maps = self.maps.iter().map(|i| i.name.as_str());
        let widget = ListPopup::new(title, maps);
        ConfigEntry::builder(widget).build(MapList(self.clone()))
    }

    fn nat_checkbox(self: &Rc<Self>) -> impl ConfigItem {
        struct B(Rc<CreateServer>);
        impl ConfigBackend<bool> for B {
            fn read(&self) -> Option<bool> {
                Some(self.0.parms.borrow().nat)
            }

            fn write(&mut self, value: bool) {
                self.0.parms.borrow_mut().nat = value;
            }
        }
        ConfigEntry::builder(Checkbox::new())
            .label(i18n::NAT_LABEL.localize())
            .hint(i18n::NAT_HINT.localize())
            .build(B(self.clone()))
    }

    fn max_players_field(self: &Rc<Self>) -> impl ConfigItem {
        struct B(Rc<CreateServer>);
        impl ConfigBackend<usize> for B {
            fn read(&self) -> Option<usize> {
                Some((self.0.parms.borrow().max_players - 1) as usize)
            }

            fn write(&mut self, value: usize) {
                self.0.parms.borrow_mut().max_players = value as u32 + 1;
            }
        }
        let title = i18n::MAX_PLAYERS_TITLE.localize();
        let items = (1..=32).map(|i| i.to_compact_string());
        let widget = ListPopup::new(title, items);
        ConfigEntry::builder(widget).build(B(self.clone()))
    }
}

pub struct CreateServerMenu {
    list: ConfigList,
}

impl CreateServerMenu {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());

        let server = CreateServer::new();
        list.add(server.start_button());
        list.add(server.server_name_field());
        list.add(server.map_list());
        list.add(server.max_players_field());
        list.add(server.password_field());
        if engine().get_cvar_float(CVAR_PUBLIC) != 0.0 {
            list.add(server.nat_checkbox());
        }

        Self { list }
    }
}

impl Menu for CreateServerMenu {
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
