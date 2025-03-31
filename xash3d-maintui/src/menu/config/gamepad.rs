use std::{
    cell::{Cell, RefCell},
    ffi::CStr,
    rc::Rc,
};

use csz::CStrArray;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::engine;

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    ui::{Control, Menu, Screen},
    widgets::ListPopup,
};

struct JoySlider {
    name: &'static CStr,
    invert: Cell<bool>,
}

impl JoySlider {
    fn new(name: &'static CStr) -> Rc<Self> {
        let invert = engine().get_cvar_float(name) < 0.0;
        Rc::new(Self {
            name,
            invert: Cell::new(invert),
        })
    }

    fn slider(self: &Rc<Self>) -> JoySliderBackend {
        JoySliderBackend {
            inner: self.clone(),
        }
    }

    fn invert(self: &Rc<Self>) -> JoyInvertBackend {
        JoyInvertBackend {
            inner: self.clone(),
        }
    }

    fn get(&self) -> f32 {
        engine().get_cvar_float(self.name).abs()
    }

    fn set(&self, value: f32) {
        let v = if self.invert.get() { -value } else { value };
        engine().set_cvar_float(self.name, v);
    }
}

struct JoySliderBackend {
    inner: Rc<JoySlider>,
}

impl ConfigBackend<f32> for JoySliderBackend {
    fn read(&self) -> Option<f32> {
        Some(self.inner.get())
    }

    fn write(&mut self, value: f32) {
        self.inner.set(value);
    }
}

struct JoyInvertBackend {
    inner: Rc<JoySlider>,
}

impl ConfigBackend<bool> for JoyInvertBackend {
    fn read(&self) -> Option<bool> {
        Some(self.inner.invert.get())
    }

    fn write(&mut self, value: bool) {
        self.inner.invert.set(value);
        self.inner.set(self.inner.get());
    }
}

fn joy_slider(list: &mut ConfigList, label: &str, cvar: &'static CStr, max: f32, step: f32) {
    let joy = JoySlider::new(cvar);
    list.add(
        ConfigEntry::slider(0.0, max, step)
            .label(label)
            .build(joy.slider()),
    );
    list.add(
        ConfigEntry::checkbox()
            .label(format!("{label} invert"))
            .build(joy.invert()),
    );
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
#[repr(usize)]
enum Axis {
    #[default]
    None = 0,
    Side,
    Forward,
    Pitch,
    Yaw,
    LeftTrigger,
    RightTrigger,
}

impl Axis {
    fn name(&self) -> &'static str {
        match self {
            Self::None => "NOT BOUND",
            Self::Side => "Side",
            Self::Forward => "Forward",
            Self::Yaw => "Yaw",
            Self::Pitch => "Pitch",
            Self::LeftTrigger => "Left Trigger",
            Self::RightTrigger => "Right Trigger",
        }
    }

    const fn all() -> &'static [Self] {
        &[
            Axis::None,
            Axis::Side,
            Axis::Forward,
            Axis::Pitch,
            Axis::Yaw,
            Axis::LeftTrigger,
            Axis::RightTrigger,
        ]
    }

    fn names() -> impl Iterator<Item = &'static str> {
        Self::all().iter().map(Self::name)
    }
}

impl From<u8> for Axis {
    fn from(c: u8) -> Self {
        match c {
            b's' => Axis::Side,
            b'f' => Axis::Forward,
            b'p' => Axis::Pitch,
            b'y' => Axis::Yaw,
            b'l' => Axis::LeftTrigger,
            b'r' => Axis::RightTrigger,
            _ => Axis::None,
        }
    }
}

impl From<Axis> for u8 {
    fn from(value: Axis) -> Self {
        match value {
            Axis::Side => b's',
            Axis::Forward => b'f',
            Axis::Pitch => b'p',
            Axis::Yaw => b'y',
            Axis::LeftTrigger => b'l',
            Axis::RightTrigger => b'r',
            Axis::None => b'0',
        }
    }
}

#[derive(Default)]
struct AxisBindingMap {
    map: RefCell<[Axis; 6]>,
}

impl AxisBindingMap {
    const CVAR_NAME: &'static CStr = c"joy_axis_binding";

    fn new() -> Rc<Self> {
        let ret = Rc::new(Self::default());
        ret.read();
        ret
    }

    fn read(&self) {
        let s = engine().get_cvar_string(Self::CVAR_NAME);
        let mut map = self.map.borrow_mut();
        for (i, c) in map.iter_mut().zip(s.bytes()) {
            *i = Axis::from(c);
        }
    }

    fn get(&self, index: usize) -> Axis {
        self.map.borrow().get(index).copied().unwrap_or_default()
    }

    fn set(&self, index: usize, value: Axis) {
        let mut map = self.map.borrow_mut();
        map[index] = value;

        let mut buf = CStrArray::<16>::new();
        let mut cur = buf.cursor();
        for &i in map.iter() {
            cur.write_bytes([u8::from(i)]).ok();
        }
        std::mem::drop(cur);
        engine().set_cvar_string(Self::CVAR_NAME, buf.as_thin());
    }

    fn config_for(self: &Rc<Self>, index: usize) -> ConfigEntry<usize, ListPopup> {
        struct B {
            map: Rc<AxisBindingMap>,
            index: usize,
        }
        impl ConfigBackend<usize> for B {
            fn read(&self) -> Option<usize> {
                Some(self.map.get(self.index) as usize)
            }

            fn write(&mut self, value: usize) {
                self.map.set(self.index, Axis::all()[value]);
            }
        }
        ConfigEntry::list(format!("Axis {index}"), Axis::names()).build(B {
            map: self.clone(),
            index,
        })
    }
}

pub struct GamepadConfig {
    list: ConfigList,
}

impl GamepadConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back("Gamepad settings");

        list.checkbox("Builtin on-screen keyboard", c"osk_enable");

        joy_slider(&mut list, "Side", c"joy_side", 1.0, 0.1);
        joy_slider(&mut list, "Forward", c"joy_forward", 1.0, 0.1);
        joy_slider(&mut list, "Look X", c"joy_pitch", 200.0, 1.0);
        joy_slider(&mut list, "Look Y", c"joy_yaw", 200.0, 1.0);

        list.label("# Axis binding map");
        let axis = AxisBindingMap::new();
        for i in 0..6 {
            list.add(axis.config_for(i));
        }

        Self { list }
    }
}

impl Menu for GamepadConfig {
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
