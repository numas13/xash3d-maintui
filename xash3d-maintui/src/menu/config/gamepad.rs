use core::{
    cell::{Cell, RefCell},
    ffi::CStr,
};

use alloc::rc::Rc;
use csz::CStrArray;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    prelude::*,
    strings::Localize,
    ui::{Control, Menu, Screen},
    widgets::ListPopup,
};

mod i18n {
    pub use crate::i18n::menu::config_gamepad::*;
}

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

fn joy_slider(
    list: &mut ConfigList,
    label: &str,
    label_invert: &str,
    cvar: &'static CStr,
    max: f32,
    step: f32,
) {
    let joy = JoySlider::new(cvar);
    list.add(
        ConfigEntry::slider(0.0, max, step)
            .label(label.localize())
            .build(joy.slider()),
    );
    list.add(
        ConfigEntry::checkbox()
            .label(label_invert.localize())
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
            Self::None => i18n::AXIS_NONE,
            Self::Side => i18n::AXIS_SIDE,
            Self::Forward => i18n::AXIS_FORWARD,
            Self::Yaw => i18n::AXIS_YAW,
            Self::Pitch => i18n::AXIS_PITCH,
            Self::LeftTrigger => i18n::AXIS_LEFT_TRIGGER,
            Self::RightTrigger => i18n::AXIS_RIGHT_TRIGGER,
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
        Self::all().iter().map(|i| i.name().localize())
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
        let engine = engine();
        let s = engine.get_cvar_string(Self::CVAR_NAME);
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
        let label = format!("{} {index}", i18n::AXIS.localize());
        ConfigEntry::list(label, Axis::names()).build(B {
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
        let mut list = ConfigList::with_back(i18n::TITLE.localize());

        list.checkbox(i18n::OSC.localize(), c"osk_enable");

        joy_slider(
            &mut list,
            i18n::SIDE,
            i18n::SIDE_INVERT,
            c"joy_side",
            1.0,
            0.1,
        );
        joy_slider(
            &mut list,
            i18n::FORWARD,
            i18n::FORWARD_INVERT,
            c"joy_forward",
            1.0,
            0.1,
        );
        joy_slider(
            &mut list,
            i18n::LOOK_X,
            i18n::LOOK_X_INVERT,
            c"joy_pitch",
            200.0,
            1.0,
        );
        joy_slider(
            &mut list,
            i18n::LOOK_Y,
            i18n::LOOK_Y_INVERT,
            c"joy_yaw",
            200.0,
            1.0,
        );

        list.label(format!("# {}", i18n::AXIS_BINDINGS_MAP.localize()));
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
