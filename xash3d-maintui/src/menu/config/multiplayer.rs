use core::{
    cell::{Cell, RefCell},
    ffi::{CStr, c_int},
    fmt::Write,
    mem,
};

use alloc::{ffi::CString, rc::Rc, vec::Vec};
use compact_str::{CompactString, ToCompactString};
use csz::{CStrArray, CStrThin};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{
    color::RGBA,
    entity::EntityType,
    ffi::common::{EF_FULLBRIGHT, kRenderNormal, vec3_t},
    misc::{Point as UiPoint, Rect as UiRect, Size as UiSize},
    picture::{Picture, PictureFlags},
    prelude::*,
    render::ViewPass,
};

use crate::{
    config_list::{ConfigBackend, ConfigEntry, ConfigList},
    input::{Key, KeyEvent},
    prelude::*,
    strings::Localize,
    ui::{
        Control, Menu, Screen,
        utils::{self, is_wide},
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
    let engine = engine();
    let files = engine.get_files_list(c"logos/*.*", false);
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
        let Ok(path) = i.to_str() else {
            warn!("invalid UTF-8 path {i}");
            continue;
        };
        let Some(name) = utils::file_stem(path) else {
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
    pic: Picture,
    color: &'static [RGBA],
}

impl LogoPreview {
    fn new(pic: Picture, color: &'static [RGBA]) -> Self {
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
        let engine = engine();
        let logo = engine.get_cvar_string(CL_LOGOFILE).to_str().unwrap();
        self.names.iter().position(|i| i.1 == logo).unwrap_or(0)
    }

    fn get_color_index(&self) -> usize {
        let engine = engine();
        let color = engine.get_cvar_string(CL_LOGOCOLOR);
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
        let pic = engine().pic_load(path).unwrap();
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
    pic: Picture,
}

impl ModelPreview {
    fn new(pic: Picture) -> Self {
        Self { pic }
    }
}

struct Model {
    engine: UiEngineRef,
    grab_input: Cell<bool>,
    names: Vec<CompactString>,
    preview: RefCell<Option<ModelPreview>>,
}

impl Model {
    fn new() -> Self {
        let engine = engine();
        let ent = engine.get_player_model_raw();
        if let Some(ent) = unsafe { ent.as_mut() } {
            let angles = vec3_t::new(0.0, 180.0, 0.0);
            ent.angles = angles;
            ent.curstate.angles = angles;
        }

        Self {
            engine,
            grab_input: Cell::new(false),
            names: get_player_models(),
            preview: RefCell::new(None),
        }
    }

    fn is_grab_input(&self) -> bool {
        self.grab_input.get()
    }

    fn get_model_name(&self) -> &CStrThin {
        self.engine.get_cvar_string(MODEL)
    }

    fn get_model_index(&self) -> usize {
        let Ok(name) = self.get_model_name().to_str() else {
            return 0;
        };
        self.names.iter().position(|i| i == name).unwrap_or(0)
    }

    fn change_animation_sequence(&self, offset: c_int) {
        let ent = engine().get_player_model_raw();
        if let Some(ent) = unsafe { ent.as_mut() } {
            let seq = ent.curstate.sequence.wrapping_add(offset).max(0);
            ent.curstate.sequence = seq;
            trace!("player model preview animation sequence is {seq}");
        }
    }

    fn rotate(&self, yaw: f32) {
        let ent = engine().get_player_model_raw();
        if let Some(ent) = unsafe { ent.as_mut() } {
            let mut yaw = ent.angles[1] + yaw * 30.0;
            if yaw >= 180.0 {
                yaw -= 360.0;
            } else if yaw <= -180.0 {
                yaw += 360.0;
            }
            ent.angles[1] = yaw;
            ent.curstate.angles[1] = yaw;
        }
    }

    fn update_entity(&self) {
        let ent = engine().get_player_model_raw();
        if ent.is_null() {
            return;
        }

        let ent = unsafe { &mut *ent };
        let old_angles = ent.angles;
        let old_sequence = ent.curstate.sequence.max(0);
        *ent = unsafe { mem::zeroed() };

        ent.index = 0;
        // draw as player model
        ent.player = 1;
        ent.curstate.body = 0;
        // IMPORTANT: always set player index to 1
        ent.curstate.number = 1;
        ent.curstate.sequence = old_sequence;
        ent.curstate.scale = 1.0;
        ent.curstate.frame = 0.0;
        ent.curstate.framerate = 1.0;
        ent.curstate.effects |= EF_FULLBRIGHT;
        ent.curstate.controller.fill(127);
        ent.latched.prevcontroller.fill(127);

        let origin = vec3_t::new(45.0, 0.0, 2.0);
        ent.origin = origin;
        ent.angles = old_angles;

        ent.curstate.origin = origin;
        ent.curstate.angles = old_angles;
    }

    fn update_preview(&self) {
        self.update_entity();

        let name = self.get_model_name();
        let engine = engine();
        let preview = engine
            .pic_load_with_flags(
                format_args!("models/player/{name}/{name}.bmp"),
                PictureFlags::KEEP_SOURCE,
            )
            .map(|pic| {
                let top_color = engine.get_cvar_float(c"topcolor");
                let bottom_color = engine.get_cvar_float(c"bottomcolor");
                engine.process_image(
                    pic.as_raw(),
                    -1.0,
                    top_color as c_int,
                    bottom_color as c_int,
                );
                ModelPreview::new(pic)
            });
        self.preview.replace(preview.ok());

        if let Some(ent) = unsafe { engine.get_player_model_raw().as_mut() } {
            if name == c"player" {
                engine.set_player_model_raw(ent, "models/player.mdl");
            } else {
                engine.set_player_model_raw(ent, format_args!("models/player/{name}/{name}.mdl"));
            }
        }
    }

    fn draw(&self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        if let Some(preview) = self.preview.borrow().as_ref() {
            Image::new(preview.pic).render(area, buf, screen);
            return;
        }

        let engine = engine();
        let px_area = screen.area_to_pixels(area);
        let pos = UiPoint::new(px_area.x.into(), px_area.y.into());
        let size = UiSize::new(px_area.width.into(), px_area.height.into());
        engine.fill_rgba(RGBA::BLACK, UiRect::from((pos, size)));

        let ent = engine.get_player_model_raw();
        if let Some(ent) = unsafe { ent.as_mut() } {
            // reset body, so it will be changed by cl_himodels setting
            ent.curstate.body = 0;

            ent.curstate.rendermode = kRenderNormal as c_int;
            ent.curstate.renderamt = 255;

            let viewpass = ViewPass::builder()
                .pos(pos.x, pos.y)
                .build(size.width as i32, size.height as i32);
            let x = 45.0 / (viewpass.fov_y() / 2.0).to_radians().tan();
            ent.origin.x = x;
            ent.curstate.origin.x = x;
            engine.clear_scene();
            engine.create_visible_entity_raw(ent, EntityType::Normal);
            engine.render_scene(viewpass);
            return;
        }

        // just draw some text if no image or model
        let name = self.get_model_name().as_c_str().to_string_lossy();
        Line::raw(name).render(area, buf);
    }

    fn key_event(&self, _backend: &XashBackend, event: KeyEvent) {
        let key = event.key();
        match key {
            Key::Mouse(0) => {
                self.change_animation_sequence(1);
            }
            Key::Mouse(1) => {
                self.change_animation_sequence(-1);
            }
            Key::TouchStart(_) => {
                self.grab_input.set(true);
            }
            Key::TouchStop(_) => {
                self.grab_input.set(false);
            }
            Key::Touch(x, _) => {
                self.rotate(-(x as f32) * 0.03);
            }
            _ => {}
        }
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
    model_area: Rect,
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

        Self {
            list,
            logo,
            model,
            model_area: Rect::default(),
        }
    }
}

impl Menu for MultiplayerConfig {
    fn vid_init(&mut self) {
        self.model.update_preview();
    }

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
            Image::with_color(preview.pic, preview.color).render(logo_area, buf, screen);
        }

        self.model_area = utils::main_block(i18n::MODEL_LABEL, model_area, buf);
        self.model.draw(self.model_area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        if self.model.is_grab_input()
            || (!self.list.is_grab_input() && backend.is_cursor_in_area(self.model_area))
        {
            self.model.key_event(backend, event);
            return Control::None;
        }

        self.list.key_event(backend, event)
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.list.mouse_event(backend)
    }
}
