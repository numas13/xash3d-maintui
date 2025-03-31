use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::KeyEvent,
    ui::{utils, Control, Menu, Screen, State},
    widgets::{ConfirmResult, Input, List, WidgetMut},
};

const MENU_TEST_A: &str = "Test A";
const MENU_TEST_B: &str = "Test B";
const MENU_TEST_C: &str = "Test C";
const MENU_RESET: &str = "Reset";
const MENU_BACK: &str = "Back";

#[derive(Copy, Clone, Default)]
enum Focus {
    #[default]
    Menu,
    Input,
}

pub struct TestMenu {
    frames: u64,
    state: State<Focus>,
    list: List,
    input: Input,
}

impl TestMenu {
    pub fn new() -> Self {
        Self {
            frames: 0,
            state: State::default(),
            list: List::new_first([MENU_TEST_A, MENU_TEST_B, MENU_TEST_C, MENU_RESET, MENU_BACK]),
            input: Input::new(),
        }
    }

    fn clear(&mut self) {
        self.frames = 0;
    }

    fn test_a(&mut self) -> Control {
        self.state.select(Focus::Input);
        Control::None
    }

    fn test_b(&mut self) -> Control {
        Control::None
    }

    fn test_c(&mut self) -> Control {
        Control::None
    }

    fn menu_exec(&mut self, i: usize) -> Control {
        match &self.list[i] {
            MENU_TEST_A => return self.test_a(),
            MENU_TEST_B => return self.test_b(),
            MENU_TEST_C => return self.test_c(),
            MENU_RESET => self.clear(),
            MENU_BACK => return Control::Back,
            item => warn!("{item} is not implemented yet"),
        }
        Control::None
    }
}

impl Menu for TestMenu {
    fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.frames += 1;
        let title = format!("Test (frame {})", self.frames);
        let area = utils::main_block(&title, area, buf);

        let [menu_area, test_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        let menu_area = utils::main_block("Menu", menu_area, buf);
        self.list.render(menu_area, buf, screen);

        let test_area = utils::main_block("Test", test_area, buf);
        let style = if matches!(self.state.focus(), Focus::Input) {
            Style::default().black().on_green()
        } else {
            Style::default()
        };
        self.input.set_style(style);
        self.input
            .show_cursor(matches!(self.state.focus(), Focus::Input));
        self.input.render(test_area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        match self.state.focus() {
            Focus::Menu => self
                .list
                .key_event(backend, event)
                .to_control(|i| self.menu_exec(i)),
            Focus::Input => {
                match self.input.key_event(backend, event) {
                    ConfirmResult::Cancel => self.state.cancel_default(),
                    ConfirmResult::Ok => self.state.confirm_default(),
                    _ => {}
                }
                Control::None
            }
        }
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        match self.state.focus() {
            Focus::Menu => self.list.mouse_event(backend),
            Focus::Input => self.input.mouse_event(backend),
        }
    }
}
