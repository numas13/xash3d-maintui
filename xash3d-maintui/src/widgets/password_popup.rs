use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{utils, Screen, State},
    widgets::{Button, ConfirmResult, Input, InputResult, WidgetMut},
};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    Cancel,
    #[default]
    Yes,
}

pub struct PasswordPopup {
    state: State<Focus>,
    title: String,
    input: Input,
    cancel: Button,
    yes: Button,
}

impl PasswordPopup {
    pub fn new(title: impl ToString) -> Self {
        Self {
            state: Default::default(),
            title: title.to_string(),
            input: Input::builder().password().build(),
            cancel: Button::cancel(),
            yes: Button::yes(),
        }
    }
}

impl WidgetMut<InputResult> for PasswordPopup {
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let area = utils::centered_rect_fixed(32, 4, area);
        let block = utils::popup_block(&self.title);
        let inner_area = block.inner(area);
        block.render(area, buf);
        let [input_area, buttons_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner_area);
        self.input
            .set_style(Style::default().white().on_dark_gray());
        self.input.show_cursor(true);
        self.input.render(input_area, buf, screen);

        let [cancel_area, yes_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(buttons_area);

        let focus = self.state.focus();
        self.cancel
            .render(cancel_area, buf, *focus == Focus::Cancel);
        self.yes.render(yes_area, buf, *focus == Focus::Yes);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> InputResult {
        let key = event.key();
        match key {
            Key::Tab => {
                if *self.state.focus() == Focus::Cancel {
                    self.state.select(Focus::Yes);
                } else {
                    self.state.select(Focus::Cancel);
                }
            }
            Key::Mouse(0) => {
                let cursor = backend.cursor_position();
                if self.cancel.area.contains(cursor) {
                    self.state.reset();
                    return InputResult::Cancel;
                } else if self.yes.area.contains(cursor) {
                    self.state.reset();
                    return InputResult::Ok(self.input.value().to_string());
                }
            }
            _ => match self.input.key_event(backend, event) {
                ConfirmResult::Ok => {
                    return match self.state.focus() {
                        Focus::Yes => InputResult::Ok(self.input.value().to_string()),
                        Focus::Cancel => InputResult::Cancel,
                    }
                }
                ConfirmResult::Cancel => return InputResult::Cancel,
                _ => {}
            },
        }
        InputResult::None
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        let cursor = backend.cursor_position();
        if self.cancel.area.contains(cursor) {
            self.state.set(Focus::Cancel);
            true
        } else if self.yes.area.contains(cursor) {
            self.state.set(Focus::Yes);
            true
        } else {
            false
        }
    }
}
