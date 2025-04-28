use core::ffi::{c_int, CStr};

use alloc::vec::Vec;
use compact_str::{CompactString, ToCompactString};
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::{
    engine, globals,
    raw::{GameInfoFlags, GameType},
};

use crate::{
    input::{Key, KeyEvent},
    menu::{self, define_menu_items},
    strings::Localize,
    ui::{utils, Control, Menu, Screen, State},
    widgets::{ConfirmPopup, ConfirmResult, List, ListPopup, SelectResult, WidgetMut},
};

mod i18n {
    pub use crate::i18n::{all::*, menu::main::*};
}

define_menu_items! {
    MENU_CONSOLE = i18n::CONSOLE, i18n::CONSOLE_HINT;
    MENU_DISCONNECT = i18n::DISCONNECT, i18n::DISCONNECT_HINT;
    MENU_RESUME_GAME = i18n::RESUME_GAME, i18n::RESUME_GAME_HINT;
    MENU_NEW_GAME = i18n::NEW_GAME, i18n::NEW_GAME_HINT;
    MENU_NEW_GAME_DEMO = i18n::NEW_GAME_DEMO, i18n::NEW_GAME_DEMO_HINT;
    MENU_HAZARD_COURSE = i18n::HAZARD_COURSE, ""; // manually created
    MENU_LOAD_GAME = i18n::LOAD_GAME, i18n::LOAD_GAME_HINT;
    MENU_SAVE_GAME = i18n::SAVE_GAME, i18n::SAVE_GAME_HINT;
    MENU_OPTIONS = i18n::OPTIONS, i18n::OPTIONS_HINT;
    MENU_INTERNET = i18n::INTERNET, i18n::INTERNET_HINT;
    MENU_LAN = i18n::LAN, i18n::LAN_HINT;
    MENU_CHANGE_GAME = i18n::CHANGE_GAME, i18n::CHANGE_GAME_HINT;
    MENU_TEST_MENU = "Test", "";
    MENU_QUIT = i18n::QUIT, i18n::QUIT_HINT;
}

const SKILL_CANCEL: &str = i18n::CANCEL;
const SKILL_EASY: &str = i18n::SKILL_EASY;
const SKILL_NORMAL: &str = i18n::SKILL_NORMAL;
const SKILL_HARD: &str = i18n::SKILL_HARD;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Menu,
    SkillSelectPopup,
    DisconnectPopup,
}

pub struct MainMenu {
    state: State<Focus>,
    menu: List,
    skill_popup: ListPopup,
    disconnect_popup: ConfirmPopup,
    has_demo: bool,
    has_hazard_course: bool,
    has_skills: bool,
    has_change_game: bool,
    start_demo: bool,
    is_client_active: bool,
    is_single: bool,
    developer: c_int,
    hint_hazard_course: CompactString,
    game_title: CompactString,
    game_type: GameType,
}

impl MainMenu {
    pub fn new() -> Self {
        let engine = engine();
        let info = engine.get_game_info_2().unwrap();
        let has_demo = engine.is_map_valid(&info.demomap);
        let title = info.title.to_str().unwrap_or("<invalid utf8>");
        let hint_hazard_course = i18n::HAZARD_COURSE_HINT
            .localize()
            .replace("{title}", title)
            .into();
        let has_skills = !info.flags.intersects(GameInfoFlags::NOSKILLS);
        let has_hazard_course =
            !info.trainmap.is_empty() && !info.trainmap.eq_ignore_case(&info.startmap);
        let has_change_game = engine.get_cvar_float("host_allow_changegame") != 0.0;

        let mut menu = List::empty();
        menu.set_bindings([
            (Key::Char(b'c'), MENU_CONSOLE),
            (Key::Char(b'd'), MENU_DISCONNECT),
            (Key::Char(b'r'), MENU_RESUME_GAME),
            (Key::Char(b'n'), MENU_NEW_GAME),
            (Key::Char(b'p'), MENU_NEW_GAME_DEMO),
            (Key::Char(b't'), MENU_HAZARD_COURSE),
            (Key::Char(b'g'), MENU_LOAD_GAME),
            (Key::Char(b's'), MENU_SAVE_GAME),
            (Key::Char(b'o'), MENU_OPTIONS),
            (Key::Char(b'i'), MENU_INTERNET),
            // TODO: key modifiers for bindings in menus?
            // (Key::Char(b''), MENU_CHANGE_GAME),
            // (Key::Char(b''), MENU_LAN),
            (Key::Char(b't'), MENU_TEST_MENU),
            (Key::Char(b'q'), MENU_QUIT),
        ]);

        Self {
            state: State::default(),
            menu,
            skill_popup: ListPopup::new(
                i18n::DIFFICULTY.localize(),
                [SKILL_CANCEL, SKILL_EASY, SKILL_NORMAL, SKILL_HARD],
            ),
            disconnect_popup: ConfirmPopup::new(i18n::DISCONNECT_POPUP.localize()),
            has_demo,
            has_hazard_course,
            has_skills,
            has_change_game,
            start_demo: false,
            is_client_active: false,
            is_single: false,
            developer: -1,
            hint_hazard_course,
            game_title: info.title.to_compact_string(),
            game_type: info.gamemode,
        }
    }

    fn is_changed(&mut self) -> bool {
        let engine = engine();
        let globals = globals();
        let is_active = engine.client_is_active();
        let is_single = globals.maxClients < 2;
        if self.is_client_active != is_active
            || self.is_single != is_single
            || self.developer != globals.developer
        {
            self.is_client_active = is_active;
            self.is_single = is_single;
            self.developer = globals.developer;
            true
        } else {
            false
        }
    }

    fn update_menu_items(&mut self) {
        if !self.is_changed() {
            return;
        }

        let selected = self.menu.state.selected().and_then(|i| self.menu.get(i));

        let mut items = Vec::with_capacity(self.menu.len());

        if self.developer != 0 {
            items.push(MENU_CONSOLE);
        }

        if self.is_client_active {
            if !self.is_single {
                items.push(MENU_DISCONNECT);
            }
            items.push(MENU_RESUME_GAME);
        }

        if self.game_type != GameType::MultiplayerOnly {
            items.push(MENU_NEW_GAME);
            if self.has_demo {
                items.push(MENU_NEW_GAME_DEMO);
            }

            if self.has_hazard_course {
                items.push(MENU_HAZARD_COURSE);
            }

            items.push(MENU_LOAD_GAME);
            if self.is_client_active && self.is_single {
                items.push(MENU_SAVE_GAME);
            }
        }

        items.push(MENU_OPTIONS);

        if self.game_type != GameType::SingleplayerOnly {
            items.push(MENU_INTERNET);
            items.push(MENU_LAN);
        }

        if self.has_change_game {
            items.push(MENU_CHANGE_GAME);
        }

        if utils::is_dev() {
            items.push(MENU_TEST_MENU);
        }
        items.push(MENU_QUIT);

        if let Some(selected) = selected {
            let i = items.iter().position(|&i| i == selected);
            self.menu.state.select(i);
        }

        self.menu.clear();
        self.menu.extend(&items);
    }

    fn get_menu_hint(&self) -> Option<&str> {
        let selected = self.menu.state.selected()?;
        match self.menu.get(selected)? {
            MENU_HAZARD_COURSE => Some(&self.hint_hazard_course),
            item => get_menu_hint(item),
        }
    }

    fn draw_menu(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.update_menu_items();
        let len = self.menu.len();
        let area = utils::menu_block(&self.game_title, area, buf);
        let area = utils::render_hint(area, buf, len, self.get_menu_hint());
        self.menu.render(area, buf, screen);
    }

    fn reset(&mut self) {
        self.menu.state.select_first();
        self.state.reset();
    }

    fn show_skill_select_popup(&mut self, is_demo: bool) {
        let n = self.skill_popup.iter().position(|i| i == SKILL_NORMAL);
        self.skill_popup.state.select(n);
        self.state.select(Focus::SkillSelectPopup);
        self.start_demo = is_demo;
    }

    fn maybe_show_skill_select_popup(&mut self, is_demo: bool) {
        if self.has_skills {
            self.show_skill_select_popup(is_demo);
        } else {
            self.start_demo = is_demo;
            self.start_new_game(1.0);
        }
    }

    fn show_disconnect_popup(&mut self) {
        self.state.select(Focus::DisconnectPopup);
    }

    fn menu_item_exec(&mut self, i: usize) -> Control {
        match &self.menu[i] {
            MENU_CONSOLE => return Control::Console,
            MENU_DISCONNECT => self.show_disconnect_popup(),
            MENU_RESUME_GAME => return Control::Hide,
            MENU_NEW_GAME => self.maybe_show_skill_select_popup(false),
            MENU_NEW_GAME_DEMO => self.maybe_show_skill_select_popup(true),
            MENU_HAZARD_COURSE => self.start_hazardcourse(),
            MENU_LOAD_GAME => return Control::Next(menu::load()),
            MENU_SAVE_GAME => return Control::Next(menu::save()),
            MENU_INTERNET => return Control::Next(menu::internet()),
            MENU_LAN => return Control::Next(menu::lan()),
            MENU_TEST_MENU => return Control::Next(menu::test()),
            MENU_OPTIONS => return Control::Next(menu::config()),
            MENU_CHANGE_GAME => return Control::Next(menu::change_game()),
            MENU_QUIT => return Control::QuitPopup,
            item => warn!("{item} is not implemented yet"),
        }
        Control::None
    }

    fn difficulty_item_exec(&mut self, i: usize) {
        let skill = match &self.skill_popup[i] {
            SKILL_CANCEL => {
                self.state.cancel_default();
                return;
            }
            SKILL_EASY => 1.0,
            SKILL_NORMAL => 2.0,
            SKILL_HARD => 3.0,
            _ => panic!(),
        };
        self.start_new_game(skill);
    }

    fn start_game(&mut self, skill: f32, cmd: &CStr, start_demo: bool) {
        self.reset();
        let eng = engine();
        eng.cvar_set(c"skill", skill);
        eng.cvar_set(c"deathmath", 0.0);
        eng.cvar_set(c"teamplay", 0.0);
        eng.cvar_set(c"coop", 0.0);
        eng.cvar_set(c"maxplayers", 1.0);
        eng.cvar_set(c"pausable", 1.0);
        eng.stop_background_track();
        if start_demo {
            let info = eng.get_game_info_2().unwrap();
            eng.client_cmd(format_args!("newgame {}", &info.demomap));
        } else {
            eng.client_cmd(cmd);
        }
    }

    fn start_hazardcourse(&mut self) {
        self.start_game(1.0, c"hazardcourse", false);
    }

    fn start_new_game(&mut self, skill: f32) {
        self.start_game(skill, c"newgame", self.start_demo);
    }

    fn disconnect(&mut self) {
        self.reset();
        engine().client_cmd(c"disconnect");
    }
}

impl Menu for MainMenu {
    fn on_menu_hide(&mut self) {
        self.state.reset();
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.draw_menu(area, buf, screen);

        match self.state.focus() {
            Focus::Menu => {}
            Focus::SkillSelectPopup => self.skill_popup.render(area, buf, screen),
            Focus::DisconnectPopup => self.disconnect_popup.render(area, buf, screen),
        }
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        match self.state.focus() {
            Focus::Menu => match self.menu.key_event(backend, event) {
                SelectResult::Ok(i) => return self.menu_item_exec(i),
                SelectResult::Cancel => return Control::QuitPopup,
                _ => {}
            },
            Focus::SkillSelectPopup => match self.skill_popup.key_event(backend, event) {
                SelectResult::Ok(i) => self.difficulty_item_exec(i),
                SelectResult::Cancel => self.state.cancel_default(),
                _ => {}
            },
            Focus::DisconnectPopup => match self.disconnect_popup.key_event(backend, event) {
                ConfirmResult::Ok => self.disconnect(),
                ConfirmResult::Cancel => self.state.deny_default(),
                _ => {}
            },
        }
        Control::None
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        match self.state.focus() {
            Focus::Menu => self.menu.mouse_event(backend),
            Focus::SkillSelectPopup => self.skill_popup.mouse_event(backend),
            Focus::DisconnectPopup => self.disconnect_popup.mouse_event(backend),
        }
    }
}
