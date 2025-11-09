use core::{
    ffi::{CStr, c_uint},
    fmt::Write,
};

use compact_str::{CompactString, ToCompactString};
use csz::{CStrArray, CStrThin};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    config_list::{CVarInvert, ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    prelude::*,
    strings::Localize,
    ui::{Control, Menu, Screen},
};

mod i18n {
    pub use crate::i18n::menu::config_video::*;
}

const FPS_VALUES: &[u16] = &[30, 60, 75, 120, 144, 244, 360, 480, 960];

fn get_renderer_name(short: &CStrThin) -> Option<CompactString> {
    let engine = engine();
    let mut buf = CStrArray::new();
    let mut name = CStrArray::new();
    for i in 0.. {
        if !engine.get_renderer(i, Some(&mut buf), Some(&mut name)) {
            break;
        }
        if short == buf.as_c_str() {
            return Some(name.to_compact_string());
        }
    }
    None
}

fn get_loaded_renderer_name() -> CompactString {
    let engine = engine();
    let short = engine.get_cvar_string(c"r_refdll_loaded");
    if !short.is_empty() {
        if let Some(name) = get_renderer_name(short) {
            return name;
        }
    }
    CompactString::new("invalid")
}

struct RemapCvar {
    name: &'static CStr,
    map: [f32; 4],
}

impl RemapCvar {
    fn new(name: &'static CStr, map: [f32; 4]) -> Self {
        Self { name, map }
    }

    fn remap(value: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
        c + (d - c) * (value - a) / (b - a)
    }
}

impl ConfigBackend<f32> for RemapCvar {
    fn read(&self) -> Option<f32> {
        let v = engine().get_cvar(self.name);
        let [a, b, c, d] = self.map;
        Some(Self::remap(v, a, b, c, d))
    }

    fn write(&mut self, value: f32) {
        let [a, b, c, d] = self.map;
        let v = Self::remap(value, c, d, a, b);
        engine().set_cvar(self.name, v);
    }
}

struct VideoMode;

impl VideoMode {
    const CVAR_NAME: &'static CStr = c"vid_mode";
}

impl ConfigBackend<usize> for VideoMode {
    fn read(&self) -> Option<usize> {
        Some(engine().get_cvar(Self::CVAR_NAME))
    }

    fn write(&mut self, mode: usize) {
        let engine = engine();
        let mut buf = CStrArray::<256>::new();
        write!(buf.cursor(), "vid_setmode {}", mode).unwrap();
        engine.client_cmd(buf.as_thin());
        engine.set_cvar(Self::CVAR_NAME, mode as f32);
    }
}

struct FpsLimit;

impl FpsLimit {
    const CVAR_NAME: &'static CStr = c"fps_max";
}

impl ConfigBackend<usize> for FpsLimit {
    fn read(&self) -> Option<usize> {
        let fps_max = engine().get_cvar_float(Self::CVAR_NAME) as u16;
        if fps_max == 0 {
            // unlimited
            return Some(FPS_VALUES.len());
        }
        for (i, fps) in FPS_VALUES.iter().enumerate().rev() {
            if *fps <= fps_max {
                return Some(i);
            }
        }
        Some(0)
    }

    fn write(&mut self, value: usize) {
        let fps = FPS_VALUES.get(value).copied().unwrap_or(0) as f32;
        engine().set_cvar_float(Self::CVAR_NAME, fps);
    }
}

struct Renderer;

impl Renderer {
    const CVAR_NAME: &'static CStr = c"r_refdll";
}

impl ConfigBackend<usize> for Renderer {
    fn read(&self) -> Option<usize> {
        let engine = engine();
        let current = engine.get_cvar_string(Self::CVAR_NAME);
        if !current.is_empty() {
            let mut buf = CStrArray::new();
            for i in 0.. {
                if !engine.get_renderer(i, Some(&mut buf), None) {
                    break;
                }
                if current == buf.as_c_str() {
                    return Some(i as usize);
                }
            }
        }
        Some(0)
    }

    fn write(&mut self, value: usize) {
        let engine = engine();
        let mut short = CStrArray::new();
        if engine.get_renderer(value as c_uint, Some(&mut short), None) {
            engine.set_cvar_string(Self::CVAR_NAME, &short);
        }
    }
}

pub struct VideoConfig {
    list: ConfigList,
}

impl VideoConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());
        list.add(
            ConfigEntry::slider_step(0.025)
                .label(i18n::GAMMA.localize())
                .build(RemapCvar::new(c"gamma", [1.8, 3.0, 0.0, 1.0])),
        );
        list.add(
            ConfigEntry::slider_step(0.025)
                .label(i18n::BRIGHTNESS.localize())
                .build(RemapCvar::new(c"brightness", [0.0, 1.0, 0.0, 3.0])),
        );
        list.add(
            ConfigEntry::list(i18n::RESOLUTION.localize(), engine().get_mode_iter())
                .build(VideoMode),
        );
        list.add(
            ConfigEntry::list(
                i18n::WINDOW_MODE.localize(),
                [
                    i18n::WINDOW_MODE_WINDOWED.localize(),
                    i18n::WINDOW_MODE_FULLSCREEN.localize(),
                    i18n::WINDOW_MODE_BORDERLESS.localize(),
                ],
            )
            .build_for_cvar(c"fullscreen"),
        );
        list.add({
            let fps_values = FPS_VALUES
                .iter()
                .map(|i| i.to_compact_string())
                .chain([i18n::FPS_UNLIMITED.localize().into()]);
            ConfigEntry::list(i18n::FPS_LIMIT.localize(), fps_values).build(FpsLimit)
        });
        list.checkbox(i18n::VSYNC.localize(), c"gl_vsync");
        list.add({
            let renderers = (0..).map_while(|i| {
                let mut name = CStrArray::new();
                if engine().get_renderer(i, None, Some(&mut name)) {
                    Some(name.to_compact_string())
                } else {
                    None
                }
            });
            ConfigEntry::list(i18n::RENDERER.localize(), renderers)
                .note(format!(
                    "({}: {})",
                    i18n::RENDERER_NOTE.localize(),
                    get_loaded_renderer_name()
                ))
                .build(Renderer)
        });
        // TODO: disable checkboxes for software renderer
        list.checkbox(i18n::DETAIL_TEXTURES.localize(), c"r_detailtextures");
        list.checkbox(i18n::USE_VBO.localize(), c"gl_vbo");
        list.checkbox(i18n::WATER_RIPPLES.localize(), c"r_ripple");
        list.checkbox(i18n::OVERBRIGHTS.localize(), c"gl_overbright");
        list.add(
            ConfigEntry::checkbox()
                .label(i18n::TEXTURE_FILTERING.localize())
                .build(CVarInvert::new(c"gl_texture_nearest")),
        );

        Self { list }
    }
}

impl Menu for VideoConfig {
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
