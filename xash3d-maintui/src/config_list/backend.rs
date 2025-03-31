use std::ffi::CStr;

use xash3d_ui::engine::CVar;

use crate::ui::utils;

pub trait ConfigBackend<V> {
    fn is_enabled(&self) -> bool {
        true
    }

    fn read(&self) -> Option<V>;

    fn write(&mut self, value: V);
}

pub struct CVarBackend {
    name: &'static CStr,
}

impl CVarBackend {
    pub fn new(name: &'static CStr) -> Self {
        Self { name }
    }
}

impl<V: CVar> ConfigBackend<V> for CVarBackend {
    fn read(&self) -> Option<V> {
        Some(utils::cvar_read(self.name))
    }

    fn write(&mut self, value: V) {
        utils::cvar_write(self.name, value);
    }
}

pub struct CVarInvert {
    name: &'static CStr,
}

impl CVarInvert {
    pub fn new(name: &'static CStr) -> Self {
        Self { name }
    }
}

impl ConfigBackend<bool> for CVarInvert {
    fn read(&self) -> Option<bool> {
        Some(!utils::cvar_read::<bool>(self.name))
    }

    fn write(&mut self, value: bool) {
        utils::cvar_write(self.name, !value);
    }
}

impl ConfigBackend<f32> for CVarInvert {
    fn read(&self) -> Option<f32> {
        Some(-utils::cvar_read::<f32>(self.name))
    }

    fn write(&mut self, value: f32) {
        utils::cvar_write(self.name, -value);
    }
}
