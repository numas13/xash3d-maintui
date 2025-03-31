use std::ffi::CStr;

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::engine;

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    ui::{Control, Menu, Screen},
    widgets::ListPopup,
};

const CMD_RATE: &[u32] = &[25, 30, 60, 100, 150, 200, 250];
const UPDATE_RATE: &[u32] = &[25, 30, 60, 100, 150, 200, 250];
const RATE: &[u32] = &[7500, 15000, 25000, 50000, 75000, 100000];

struct Network {
    name: &'static str,
    cmd_rate: u32,
    update_rate: u32,
    rate: u32,
}

impl Network {
    const fn new(
        name: &'static str,
        _max_packet: u32,
        _max_payload: u32,
        cmd_rate: u32,
        update_rate: u32,
        rate: u32,
    ) -> Self {
        Self {
            name,
            cmd_rate,
            update_rate,
            rate,
        }
    }
}

const NETWORKS: &[Network] = &[
    Network::new("Normal internet connection", 1400, 0, 30, 60, 25000),
    Network::new(
        "DSL or PPTP with limited packet size",
        1200,
        1000,
        30,
        60,
        25000,
    ),
    Network::new("Slow connection mode (64kbps)", 900, 700, 25, 30, 7500),
];

fn cvar_to_index(name: &CStr, slice: &[u32]) -> usize {
    let current = engine().get_cvar_float(name) as u32;
    slice
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, v)| if *v <= current { Some(i) } else { None })
        .unwrap_or(0)
}

struct NetworkMode;

impl NetworkMode {
    fn config() -> ConfigEntry<usize, ListPopup> {
        ConfigEntry::list("Network mode", NETWORKS.iter().map(|i| i.name))
            .fixed_value("Select")
            .build(Self)
    }
}

impl ConfigBackend<usize> for NetworkMode {
    fn read(&self) -> Option<usize> {
        None
    }

    fn write(&mut self, value: usize) {
        let eng = engine();
        let network = &NETWORKS[value];
        eng.set_cvar_float(c"cl_cmdrate", network.cmd_rate as f32);
        eng.set_cvar_float(c"cl_updaterate", network.update_rate as f32);
        eng.set_cvar_float(c"rate", network.rate as f32);
    }
}

struct Rate {
    cvar: &'static CStr,
    list: &'static [u32],
    enabled: bool,
}

impl Rate {
    fn config(
        name: &str,
        cvar: &'static CStr,
        list: &'static [u32],
        enabled: bool,
    ) -> ConfigEntry<usize, ListPopup> {
        ConfigEntry::list(name, list).build(Rate {
            cvar,
            list,
            enabled,
        })
    }
}

impl ConfigBackend<usize> for Rate {
    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn read(&self) -> Option<usize> {
        Some(cvar_to_index(self.cvar, self.list))
    }

    fn write(&mut self, value: usize) {
        engine().set_cvar_float(self.cvar, self.list[value] as f32);
    }
}

pub struct NetworkConfig {
    list: ConfigList,
}

impl NetworkConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back("Network settings");
        list.checkbox("Allow download", c"cl_allowdownload");
        list.add(NetworkMode::config());
        let devel = engine().get_cvar_float(c"developer") as i32;
        list.add(Rate::config("Network speed", c"rate", RATE, devel > 0));
        list.add(Rate::config(
            "Command rate",
            c"cl_cmdrate",
            CMD_RATE,
            devel > 1,
        ));
        list.add(Rate::config(
            "Update rate",
            c"cl_updaterate",
            UPDATE_RATE,
            devel > 1,
        ));
        Self { list }
    }
}

impl Menu for NetworkConfig {
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
