use std::{
    cell::RefCell,
    ffi::{c_int, CStr, CString},
    fmt::Write,
    path::Path,
    rc::Rc,
};

use compact_str::{CompactString, ToCompactString};
use csz::{CStrArray, CStrThin};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{
    color::RGBA,
    engine,
    picture::Picture,
    raw::{PictureFlags, HIMAGE},
};

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::KeyEvent,
    strings::Localize,
    ui::{
        utils::{self, is_wide},
        Control, Menu, Screen,
    },
    widgets::{Image, Slider, WidgetMut},
};

mod i18n {
    pub use crate::i18n::menu::config_multiplayer::*;
}

const CL_LOGOFILE: &CStr = c"cl_logofile";
const CL_LOGOCOLOR: &CStr = c"cl_logocolor";

const MODEL: &CStr = c"model";

const ORANGE: RGBA = RGBA::rgb(255, 120, 24);
const YELLOW: RGBA = RGBA::rgb(225, 180, 24);
const BLUE: RGBA = RGBA::rgb(0, 60, 255);
const LTBLUE: RGBA = RGBA::rgb(0, 167, 255);
const GREEN: RGBA = RGBA::rgb(0, 167, 0);
const RED: RGBA = RGBA::rgb(255, 43, 0);
const BROWN: RGBA = RGBA::rgb(123, 73, 0);
const LTGRAY: RGBA = RGBA::rgb(100, 100, 100);
const DKGRAY: RGBA = RGBA::rgb(36, 36, 36);
const COLOR_RAINBOW: &[RGBA] = &[
    RGBA::rgb(0xE4, 0x03, 0x03),
    RGBA::rgb(0xFF, 0x8C, 0x00),
    RGBA::rgb(0xFF, 0xED, 0x00),
    RGBA::rgb(0x00, 0x80, 0x26),
    RGBA::rgb(0x24, 0x40, 0x8E),
    RGBA::rgb(0x73, 0x29, 0x82),
];
const COLOR_LESBIAN: &[RGBA] = &[
    RGBA::rgb(0xD5, 0x2D, 0x00),
    RGBA::rgb(0xEF, 0x76, 0x27),
    RGBA::rgb(0xFF, 0x9A, 0x56),
    RGBA::rgb(0xFF, 0xFF, 0xFF),
    RGBA::rgb(0xD1, 0x62, 0xA4),
    RGBA::rgb(0xB5, 0x56, 0x90),
    RGBA::rgb(0xA3, 0x02, 0x62),
];
const COLOR_GAY: &[RGBA] = &[
    RGBA::rgb(0x07, 0x8D, 0x70),
    RGBA::rgb(0x26, 0xCE, 0xAA),
    RGBA::rgb(0x98, 0xE8, 0xC1),
    RGBA::rgb(0xFF, 0xFF, 0xFF),
    RGBA::rgb(0x7B, 0xAD, 0xE2),
    RGBA::rgb(0x50, 0x49, 0xCC),
    RGBA::rgb(0x3D, 0x1A, 0x78),
];
const COLOR_BI: &[RGBA] = &[
    RGBA::rgb(0xD6, 0x02, 0x70),
    RGBA::rgb(0xD6, 0x02, 0x70),
    RGBA::rgb(0x9B, 0x4F, 0x96),
    RGBA::rgb(0x00, 0x38, 0xA8),
    RGBA::rgb(0x00, 0x38, 0xA8),
];
const COLOR_TRANS: &[RGBA] = &[
    RGBA::rgb(0x5B, 0xCE, 0xFA),
    RGBA::rgb(0xF5, 0xA9, 0xB8),
    RGBA::rgb(0xFF, 0xFF, 0xFF),
    RGBA::rgb(0xF5, 0xA9, 0xB8),
    RGBA::rgb(0x5B, 0xCE, 0xFA),
];
const COLOR_PAN: &[RGBA] = &[
    RGBA::rgb(0xFF, 0x21, 0x8C),
    RGBA::rgb(0xFF, 0xD8, 0x00),
    RGBA::rgb(0x21, 0xB1, 0xFF),
];
const COLOR_ENBY: &[RGBA] = &[
    RGBA::rgb(0xFC, 0xF4, 0x34),
    RGBA::rgb(0xFF, 0xFF, 0xFF),
    RGBA::rgb(0x9C, 0x59, 0xD1),
    RGBA::rgb(0x2C, 0x2C, 0x2C),
];

struct LogoColor {
    name: CompactString,
    value: &'static CStr,
    colors: &'static [RGBA],
}

impl LogoColor {
    fn new(name: &str, value: &'static CStr, colors: &'static [RGBA]) -> Self {
        Self {
            name: name.localize().into(),
            value,
            colors,
        }
    }
}

fn get_logo_list() -> Vec<(CString, CompactString)> {
    let files = engine().get_files_list(c"logos/*.*", false);
    let mut list = vec![];
    for i in files.iter() {
        let Ok(path) = i.to_str() else {
            warn!("invalid UTF-8 path {i}");
            continue;
        };
        let Some(ext) = utils::file_extension(path) else {
            continue;
        };
        if !["bmp", "png"].iter().any(|i| ext.eq_ignore_ascii_case(i)) {
            continue;
        }
        let Some(name) = utils::file_stem(path) else {
            continue;
        };
        if name.eq_ignore_ascii_case("remapped") {
            continue;
        }
        list.push((i.into(), name.into()));
    }
    list
}

fn get_logo_color_list() -> Vec<LogoColor> {
    let mut colors = vec![];
    let mut f = |n, v, c| colors.push(LogoColor::new(n, v, c));
    f("FullColor", c"FullColor", &[]);
    f("#Valve_Orange", c"Orange", &[ORANGE]);
    f("#Valve_Yellow", c"Yellow", &[YELLOW]);
    f("#Valve_Blue", c"Blue", &[BLUE]);
    f("#Valve_Ltblue", c"Ltblue", &[LTBLUE]);
    f("#Valve_Green", c"Green", &[GREEN]);
    f("#Valve_Red", c"Red", &[RED]);
    f("#Valve_Brown", c"Brown", &[BROWN]);
    f("#Valve_Ltgray", c"Ltgray", &[LTGRAY]);
    f("#Valve_Dkgray", c"Dkgray", &[DKGRAY]);
    f("Rainbow", c"Rainbow", COLOR_RAINBOW);
    f("Lesbian", c"Lesbian", COLOR_LESBIAN);
    f("Gay", c"Gay", COLOR_GAY);
    f("Bi", c"Bi", COLOR_BI);
    f("Trans", c"Trans", COLOR_TRANS);
    f("Pan", c"Pan", COLOR_PAN);
    f("Enby", c"Enby", COLOR_ENBY);
    colors
}

fn get_player_models() -> Vec<CompactString> {
    let engine = engine();
    let files = engine.get_files_list(c"models/player/*", false);
    let mut list = vec![];
    let mut buf = CStrArray::<512>::new();
    for i in files.iter() {
        let Ok(path) = i.to_str().map(Path::new) else {
            warn!("invalid UTF-8 path {i}");
            continue;
        };
        let Some(name) = path.file_stem().and_then(|i| i.to_str()) else {
            continue;
        };
        write!(buf.cursor(), "models/player/{name}/{name}.mdl").unwrap();
        if engine.file_exists(buf.as_c_str(), false) {
            list.push(name.into());
        }
    }
    list
}

struct LogoPreview {
    pic: Picture<CString>,
    color: &'static [RGBA],
}

impl LogoPreview {
    fn new(pic: Picture<CString>, color: &'static [RGBA]) -> Self {
        Self { pic, color }
    }
}

struct Logo {
    names: Vec<(CString, CompactString)>,
    colors: Vec<LogoColor>,
    preview: RefCell<Option<LogoPreview>>,
}

impl Logo {
    fn new() -> Self {
        Self {
            names: get_logo_list(),
            colors: get_logo_color_list(),
            preview: Default::default(),
        }
    }

    fn get_logo_index(&self) -> usize {
        let logo = engine().get_cvar_string(CL_LOGOFILE).to_str().unwrap();
        self.names.iter().position(|i| i.1 == logo).unwrap_or(0)
    }

    fn get_color_index(&self) -> usize {
        let color = engine().get_cvar_string(CL_LOGOCOLOR);
        self.colors
            .iter()
            .position(|i| i.value == color)
            .unwrap_or(0)
    }

    fn get_path(&self, i: usize) -> Option<&CStr> {
        self.names.get(i).map(|i| i.0.as_c_str())
    }

    fn get_color(&self, i: usize) -> &'static [RGBA] {
        self.colors.get(i).map_or(&[][..], |i| i.colors)
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(|i| i.1.as_str())
    }

    fn colors(&self) -> impl Iterator<Item = &str> {
        self.colors.iter().map(|i| i.name.as_str())
    }

    fn set_logo(&self, i: usize) {
        if let Some(logo) = self.names.get(i) {
            engine().set_cvar_string(CL_LOGOFILE, logo.1.as_str());
            self.update_preview();
        }
    }

    fn set_color(&self, i: usize) {
        if let Some(color) = self.colors.get(i) {
            engine().set_cvar_string(CL_LOGOCOLOR, color.value);
            self.update_preview();
        }
    }

    fn update_preview(&self) {
        let Some(path) = self.get_path(self.get_logo_index()) else {
            return;
        };
        let pic = Picture::<CString>::load(path.into(), PictureFlags::empty());
        let color = self.get_color(self.get_color_index());
        *self.preview.borrow_mut() = Some(LogoPreview::new(pic, color));
    }
}

struct LogoConfig(Rc<Logo>);

impl ConfigBackend<usize> for LogoConfig {
    fn read(&self) -> Option<usize> {
        Some(self.0.get_logo_index())
    }

    fn write(&mut self, value: usize) {
        self.0.set_logo(value);
    }
}

struct LogoColorConfig(Rc<Logo>);

impl ConfigBackend<usize> for LogoColorConfig {
    fn read(&self) -> Option<usize> {
        Some(self.0.get_color_index())
    }

    fn write(&mut self, value: usize) {
        self.0.set_color(value);
    }
}

struct ModelPreview {
    pic: HIMAGE,
}

impl ModelPreview {
    fn new(pic: HIMAGE) -> Self {
        Self { pic }
    }
}

struct Model {
    names: Vec<CompactString>,
    preview: RefCell<Option<ModelPreview>>,
}

impl Model {
    fn new() -> Self {
        Self {
            names: get_player_models(),
            preview: RefCell::new(None),
        }
    }

    fn get_model_name(&self) -> &CStrThin {
        engine().get_cvar_string(MODEL)
    }

    fn get_model_index(&self) -> usize {
        let Ok(name) = self.get_model_name().to_str() else {
            return 0;
        };
        self.names.iter().position(|i| i == name).unwrap_or(0)
    }

    fn update_preview(&self) {
        let name = self.get_model_name();
        let mut path = CStrArray::<512>::new();
        write!(path.cursor(), "models/player/{name}/{name}.bmp").unwrap();
        let engine = engine();
        let pic = engine.pic_load(path.as_c_str(), None, PictureFlags::KEEP_SOURCE.bits());
        let preview = if pic != 0 {
            let top_color = engine.get_cvar_float(c"topcolor");
            let bottom_color = engine.get_cvar_float(c"bottomcolor");
            engine.process_image(pic, -1.0, top_color as c_int, bottom_color as c_int);
            Some(ModelPreview::new(pic))
        } else {
            None
        };
        self.preview.replace(preview);
    }
}

struct ModelConfig(Rc<Model>);

impl ConfigBackend<usize> for ModelConfig {
    fn read(&self) -> Option<usize> {
        Some(self.0.get_model_index())
    }

    fn write(&mut self, value: usize) {
        if let Some(model) = self.0.names.get(value) {
            engine().set_cvar_string(MODEL, model.as_str());
            self.0.update_preview();
        }
    }
}

struct ModelColorConfig {
    name: &'static CStr,
    model: Rc<Model>,
}

impl ConfigBackend<f32> for ModelColorConfig {
    fn read(&self) -> Option<f32> {
        Some(engine().get_cvar_float(self.name))
    }

    fn write(&mut self, value: f32) {
        engine().set_cvar_float(self.name, value);
        self.model.update_preview();
    }
}

struct PlayerName;

impl ConfigBackend<CompactString> for PlayerName {
    fn read(&self) -> Option<CompactString> {
        Some(engine().get_cvar_string(c"name").to_compact_string())
    }

    fn write(&mut self, value: CompactString) {
        engine().set_cvar_string(c"name", value.as_str());
    }
}

pub struct MultiplayerConfig {
    list: ConfigList,
    logo: Rc<Logo>,
    model: Rc<Model>,
}

impl MultiplayerConfig {
    pub fn new() -> Self {
        let mut list = ConfigList::with_back(i18n::TITLE.localize());

        list.add(
            ConfigEntry::input()
                .label(i18n::PLAYER_NAME.localize())
                .hint(i18n::PLAERY_NAME_HINT.localize())
                .build(PlayerName),
        );

        let logo = Rc::new(Logo::new());
        logo.update_preview();
        list.add(
            ConfigEntry::list(i18n::LOGO_TITLE.localize(), logo.names())
                .label(i18n::LOGO_LABEL.localize())
                .hint(i18n::LOGO_HINT.localize())
                .build(LogoConfig(logo.clone())),
        );

        list.add(
            ConfigEntry::list(i18n::COLOR_TITLE.localize(), logo.colors())
                .label(i18n::COLOR_LABEL.localize())
                .hint(i18n::COLOR_HINT.localize())
                .build(LogoColorConfig(logo.clone())),
        );

        let model = Rc::new(Model::new());
        model.update_preview();
        list.add(
            ConfigEntry::list(i18n::MODEL_TITLE.localize(), &model.names)
                .label(i18n::MODEL_LABEL.localize())
                .hint(i18n::MODEL_HINT.localize())
                .build(ModelConfig(model.clone())),
        );

        let top_color = (c"topcolor", i18n::TOP_COLOR, i18n::TOP_COLOR_HINT);
        let bottom_color = (c"bottomcolor", i18n::BOTTOM_COLOR, i18n::BOTTOM_COLOR_HINT);
        for (name, label, hint) in [top_color, bottom_color] {
            let widget = Slider::builder().max(255.0).step(1.0).build();
            let entry = ConfigEntry::builder(widget)
                .label(label.localize())
                .hint(hint.localize())
                .build(ModelColorConfig {
                    name,
                    model: model.clone(),
                });
            list.add(entry)
        }

        list.add({
            ConfigEntry::checkbox()
                .label(i18n::HIGH_MODELS.localize())
                .hint(i18n::HIGH_MODELS_HINT.localize())
                .build_for_cvar(c"cl_himodels")
        });

        Self { list, logo, model }
    }
}

impl Menu for MultiplayerConfig {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let constraints = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let layout = if is_wide(area) {
            Layout::horizontal(constraints)
        } else {
            Layout::vertical(constraints)
        };
        let [list_area, images_area] = layout.areas(area);

        self.list.draw(list_area, buf, screen);

        let layout = if is_wide(area) {
            Layout::vertical(constraints)
        } else {
            Layout::horizontal(constraints)
        };
        let [logo_area, model_area] = layout.areas(images_area);

        let logo_area = utils::main_block(i18n::LOGO_LABEL, logo_area, buf);
        if let Some(preview) = self.logo.preview.borrow().as_ref() {
            Image::with_color(preview.pic.raw(), preview.color).render(logo_area, buf, screen);
        }

        let model_area = utils::main_block(i18n::MODEL_LABEL, model_area, buf);
        if let Some(preview) = self.model.preview.borrow().as_ref() {
            Image::new(preview.pic).render(model_area, buf, screen);
        } else if let Ok(name) = self.model.get_model_name().to_str() {
            Line::raw(name).render(model_area, buf);
        }
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        self.list.key_event(backend, event)
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.list.mouse_event(backend)
    }
}
