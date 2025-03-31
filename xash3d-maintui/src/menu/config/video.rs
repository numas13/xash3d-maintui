use std::{
    ffi::{c_uint, CStr},
    fmt::Write,
};

use csz::{CStrArray, CStrThin};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::engine;

use crate::{
    config_list::{CVarInvert, ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    ui::{utils, Control, Menu, Screen},
};

const FPS_VALUES: &[u16] = &[30, 60, 75, 120, 144, 244, 360, 480, 960];

fn get_renderer_name(short: &CStrThin) -> Option<String> {
    let engine = engine();
    let mut buf = CStrArray::new();
    let mut name = CStrArray::new();
    for i in 0.. {
        if !engine.get_renderer(i, Some(&mut buf), Some(&mut name)) {
            break;
        }
        if short == buf.as_c_str() {
            return Some(name.to_string());
        }
    }
    None
}

fn get_loaded_renderer_name() -> String {
    let short = engine().get_cvar_string(c"r_refdll_loaded");
    if !short.is_empty() {
        if let Some(name) = get_renderer_name(short) {
            return name;
        }
    }
    String::from("invalid")
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
        let v = engine().cvar(self.name);
        let [a, b, c, d] = self.map;
        Some(Self::remap(v, a, b, c, d))
    }

    fn write(&mut self, value: f32) {
        let [a, b, c, d] = self.map;
        let v = Self::remap(value, c, d, a, b);
        engine().cvar_set(self.name, v);
    }
}

struct VideoMode;

impl VideoMode {
    const CVAR_NAME: &'static CStr = c"vid_mode";
}

impl ConfigBackend<usize> for VideoMode {
    fn read(&self) -> Option<usize> {
        Some(utils::cvar_read(Self::CVAR_NAME))
    }

    fn write(&mut self, mode: usize) {
        let engine = engine();
        let mut buf = CStrArray::<256>::new();
        write!(buf.cursor(), "vid_setmode {}", mode).unwrap();
        engine.client_cmd(buf.as_thin());
        engine.cvar_set(Self::CVAR_NAME, mode as f32);
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
        let mut list = ConfigList::with_back("Video settings");
        list.add(
            ConfigEntry::slider_step(0.025)
                .label("Gamma")
                .build(RemapCvar::new(c"gamma", [1.8, 3.0, 0.0, 1.0])),
        );
        list.add(
            ConfigEntry::slider_step(0.025)
                .label("Brightness")
                .build(RemapCvar::new(c"brightness", [0.0, 1.0, 0.0, 3.0])),
        );
        list.add(ConfigEntry::list("Resolution", engine().get_mode_iter()).build(VideoMode));
        list.add(
            ConfigEntry::list("Window Mode", ["Windowed", "Fullscreen", "Borderless"])
                .build_for_cvar(c"fullscreen"),
        );
        list.add({
            let fps_values = FPS_VALUES
                .iter()
                .map(|i| i.to_string())
                .chain(["Unlimited".to_string()]);
            ConfigEntry::list("FPS limit", fps_values).build(FpsLimit)
        });
        list.checkbox("VSync", c"gl_vsync");
        list.add({
            let renderers = (0..).map_while(|i| {
                let mut name = CStrArray::new();
                if engine().get_renderer(i, None, Some(&mut name)) {
                    Some(name.to_string())
                } else {
                    None
                }
            });
            ConfigEntry::list("Renderer", renderers)
                .note(format!("(loaded: {})", get_loaded_renderer_name()))
                .build(Renderer)
        });
        // TODO: disable checkboxes for software renderer
        list.checkbox("Detail textures", c"r_detailtextures");
        list.checkbox("Use VBO", c"gl_vbo");
        list.checkbox("Water ripples", c"r_ripple");
        list.checkbox("Overbrights", c"gl_overbright");
        list.add(
            ConfigEntry::checkbox()
                .label("Texture filtering")
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
