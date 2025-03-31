use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{utils, Screen, State},
    widgets::{Button, ConfirmResult, WidgetMut},
};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
enum Focus {
    #[default]
    Cancel,
    Yes,
}

pub struct ConfirmPopup {
    state: State<Focus>,
    cancel: Button,
    yes: Button,
    content: String,
}

impl ConfirmPopup {
    pub fn new(content: impl ToString) -> Self {
        let mut content = content.to_string();
        content.push_str(" (y/n)");
        Self {
            state: Default::default(),
            cancel: Button::cancel(),
            yes: Button::yes(),
            content,
        }
    }
}

impl WidgetMut<ConfirmResult> for ConfirmPopup {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _: &Screen) {
        let width = 2 + self.content.len() as u16;
        let area = utils::centered_rect_fixed(width, 4, area);

        let block = utils::popup_block("Y/N");
        let inner_area = block.inner(area);
        block.render(area, buf);

        let [text_area, buttons_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Length(1)]).areas(inner_area);

        let t = Text::styled(&*self.content, Style::new().red().on_gray());
        let p = Paragraph::new(t).wrap(Wrap { trim: true });
        p.render(text_area, buf);

        let [cancel_area, yes_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(buttons_area);

        let focus = self.state.focus();
        self.cancel
            .render(cancel_area, buf, *focus == Focus::Cancel);
        self.yes.render(yes_area, buf, *focus == Focus::Yes);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let mut ret = ConfirmResult::None;
        match event.key() {
            Key::Enter => match self.state.focus() {
                Focus::Cancel => ret = ConfirmResult::Cancel,
                Focus::Yes => ret = ConfirmResult::Ok,
            },
            Key::Tab => {
                if *self.state.focus() == Focus::Cancel {
                    self.state.select(Focus::Yes);
                } else {
                    self.state.select(Focus::Cancel);
                }
            }
            Key::Char(b'n') => ret = ConfirmResult::Cancel,
            Key::Char(b'y') => ret = ConfirmResult::Ok,
            Key::Char(b'h') | Key::ArrowLeft => {
                self.state.select(Focus::Cancel);
            }
            Key::Char(b'l') | Key::ArrowRight => {
                self.state.select(Focus::Yes);
            }
            Key::Mouse(0) => {
                let cursor = backend.cursor_position();
                if self.cancel.area.contains(cursor) {
                    ret = ConfirmResult::Cancel;
                } else if self.yes.area.contains(cursor) {
                    ret = ConfirmResult::Ok;
                }
            }
            _ => {}
        }
        if ret != ConfirmResult::None {
            self.state.reset();
        }
        ret
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
