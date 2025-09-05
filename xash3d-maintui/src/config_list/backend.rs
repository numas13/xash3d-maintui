use core::ffi::CStr;

use xash3d_ui::{
    cvar::{GetCvar, SetCvar},
    prelude::*,
};

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

impl<'a, V: GetCvar<'a> + SetCvar> ConfigBackend<V> for CVarBackend {
    fn read(&self) -> Option<V> {
        Some(engine().get_cvar(self.name))
    }

    fn write(&mut self, value: V) {
        engine().set_cvar(self.name, value);
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
        Some(!engine().get_cvar::<bool>(self.name))
    }

    fn write(&mut self, value: bool) {
        engine().set_cvar(self.name, !value);
    }
}

impl ConfigBackend<f32> for CVarInvert {
    fn read(&self) -> Option<f32> {
        Some(-engine().get_cvar::<f32>(self.name))
    }

    fn write(&mut self, value: f32) {
        engine().set_cvar(self.name, -value);
    }
}
