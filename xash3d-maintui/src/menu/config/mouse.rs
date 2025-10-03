use core::ffi::CStr;

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    prelude::*,
    strings::Localize,
    ui::{Control, Menu, Screen},
    widgets::Checkbox,
};

mod i18n {
    pub use crate::i18n::menu::config_mouse::*;
}

struct MouseInvert;

impl MouseInvert {
    const M_PITCH: &'static CStr = c"m_pitch";

    fn config() -> ConfigEntry<bool, Checkbox> {
        ConfigEntry::checkbox()
            .label(i18n::INVERT_MOUSE.localize())
            .build(Self)
    }
}

impl ConfigBackend<bool> for MouseInvert {
    fn read(&self) -> Option<bool> {
        Some(engine().get_cvar::<f32>(Self::M_PITCH) < 0.0)
    }

    fn write(&mut self, value: bool) {
        let engine = engine();
        let p = engine.get_cvar::<f32>(Self::M_PITCH).abs();
        engine.set_cvar(Self::M_PITCH, if value { -p } else { p })
    }
}

struct MouseLook;

impl MouseLook {
    fn config() -> ConfigEntry<bool, Checkbox> {
        ConfigEntry::checkbox()
            .label(i18n::MOUSE_LOOK.localize())
            .build(Self)
    }
}

impl ConfigBackend<bool> for MouseLook {
    fn is_enabled(&self) -> bool {
        engine().key_get_state(c"in_mlook").is_some()
    }

    fn read(&self) -> Option<bool> {
        Some(
            engine()
                .key_get_state(c"in_mlook")
                .map_or(true, |i| i.is_down()),
        )
    }

    fn write(&mut self, value: bool) {
        let engine = engine();
        let cmd = if value { c"+mlook\n" } else { c"-mlook\n" };
        engine.client_cmd(cmd);
        // FIXME:
        // engine.cvar_set(c"lookspring", 0.0);
        // engine.cvar_set(c"lookstrafe", 0.0);
    }
}

struct Look {
    name: &'static CStr,
}

impl Look {
    fn config(label: &str, name: &'static CStr) -> ConfigEntry<bool, Checkbox> {
        ConfigEntry::checkbox().label(label).build(Look { name })
    }
}

impl ConfigBackend<bool> for Look {
    fn is_enabled(&self) -> bool {
        // TODO: store in_mlook in global state
        !engine()
            .key_get_state(c"in_mlook")
            .is_some_and(|i| i.is_down())
    }

    fn read(&self) -> Option<bool> {
        Some(engine().get_cvar(self.name))
    }

    fn write(&mut self, value: bool) {
        engine().set_cvar(self.name, value);
    }
}

pub struct MouseConfig {
    list: ConfigList,
}

impl MouseConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());
        list.checkbox(i18n::CROSSHAIR.localize(), c"crosshair");
        list.add(MouseInvert::config());
        list.add(MouseLook::config());
        list.add(Look::config(i18n::LOOK_SPRING.localize(), c"lookspring"));
        list.add(Look::config(i18n::LOOK_STRAFE.localize(), c"lookstrafe"));
        list.checkbox(i18n::MOUSE_FILTER.localize(), c"look_filter");
        list.checkbox(i18n::AUTO_AIM.localize(), c"sv_aim");
        list.checkbox(i18n::RAW_INPUT.localize(), c"m_rawinput");
        list.add(
            ConfigEntry::slider(0.0, 20.0, 0.1)
                .label(i18n::AIM_SENSITIVITY.localize())
                .build_for_cvar(c"sensitivity"),
        );
        Self { list }
    }
}

impl Menu for MouseConfig {
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
