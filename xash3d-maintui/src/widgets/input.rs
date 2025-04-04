use std::cmp;

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;
use xash3d_ui::engine;

use crate::{
    input::{Key, KeyEvent},
    ui::Screen,
    widgets::{ConfirmResult, Value, WidgetMut},
};

const SPACES: &str = "                                                                ";
const PASSWORD: &str = "****************************************************************";

pub struct InputBuilder(Input);

impl InputBuilder {
    pub fn password(mut self) -> Self {
        self.0.password = true;
        self
    }

    pub fn build(self) -> Input {
        self.0
    }
}

pub struct Input {
    cursor: u16,
    style: Style,
    value: String,
    password: bool,
    show_cursor: bool,
}

impl Input {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> InputBuilder {
        InputBuilder(Self {
            cursor: 0,
            style: Style::default(),
            value: String::new(),
            password: false,
            show_cursor: false,
        })
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    // pub fn set_value(&mut self, value: &str) {
    //     self.value.clear();
    //     self.value.push_str(value);
    // }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn show_cursor(&mut self, show: bool) {
        self.show_cursor = show;
    }

    fn cursor_to_offset(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor as usize)
            .map_or(self.value.len(), |i| i.0)
    }

    pub fn cursor_left(&mut self, n: u16) {
        self.cursor = self.cursor.saturating_sub(n);
    }

    pub fn cursor_right(&mut self, n: u16) {
        self.cursor = cmp::min(self.cursor + n, self.value.chars().count() as u16);
    }

    pub fn cursor_to_start(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_to_end(&mut self) {
        self.cursor = self.value.chars().count() as u16
    }

    pub fn push(&mut self, c: char) {
        self.value.insert(self.cursor_to_offset(), c);
        self.cursor += 1;
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
}

impl WidgetMut<ConfirmResult> for Input {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _: &Screen) {
        let mut s = self.value.as_str();
        if self.password {
            let width = self.value.chars().count();
            s = &PASSWORD[..width];
        }
        let style = if self.show_cursor {
            Style::default().white().on_dark_gray()
        } else {
            self.style
        };
        Line::raw(SPACES).style(style).render(area, buf);
        Line::raw(s).style(style).render(area, buf);

        if self.show_cursor && self.cursor < area.width {
            let offset = self.cursor_to_offset();
            let cursor_area = Rect {
                x: area.x + self.cursor,
                width: 1,
                ..area
            };
            let mut s = &self.value[offset..];
            if s.is_empty() {
                s = " ";
            }
            Line::raw(s)
                .style(Style::default().on_red())
                .render(cursor_area, buf);
        }
    }

    fn key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        match key {
            Key::Enter => return ConfirmResult::Ok,
            Key::Escape => return ConfirmResult::Cancel,
            Key::Home => self.cursor_to_start(),
            Key::End => self.cursor_to_end(),
            Key::ArrowLeft => self.cursor_left(1),
            Key::ArrowRight => self.cursor_right(1),
            Key::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor_to_offset());
                }
            }
            Key::Delete => {
                let offset = self.cursor_to_offset();
                if offset < self.value.len() {
                    self.value.remove(offset);
                }
            }
            _ if event.alt() => {}
            _ if event.ctrl() => match key {
                Key::Char(b'a') => self.cursor_to_start(),
                Key::Char(b'e') => self.cursor_to_end(),
                Key::Char(b'w') => {
                    let end = self.cursor_to_offset();
                    let start = self.value[..end]
                        .char_indices()
                        .rev()
                        .find(|i| i.1.is_ascii_whitespace())
                        .map_or(0, |i| i.0);
                    self.cursor -= self.value.drain(start..end).count() as u16;
                }
                Key::Char(b'u') => {
                    self.value.drain(0..self.cursor_to_offset()).count();
                    self.cursor = 0;
                }
                Key::Char(b'v') => {
                    if let Some(s) = engine().get_clipboard_data() {
                        for c in s.chars() {
                            self.push(c);
                        }
                    }
                }
                _ => {}
            },
            Key::Char(c) if c.is_ascii_whitespace() || c.is_ascii_graphic() => {
                let c = match c {
                    _ if event.shift() => match c {
                        c if c.is_ascii_alphabetic() => c.to_ascii_uppercase(),
                        c if c.is_ascii_digit() => b"!@#$%^&*()"[(c - b'0') as usize],
                        b'-' => b'_',
                        b'=' => b'+',
                        b'\\' => b'|',
                        b'[' => b'{',
                        b']' => b'}',
                        b'\'' => b'"',
                        b',' => b'<',
                        b'.' => b'>',
                        b'/' => b'?',
                        b'`' => b'~',
                        b';' => b':',
                        _ => c,
                    },
                    _ => c,
                };
                self.push(char::from(c));
            }
            _ => {}
        }
        ConfirmResult::None
    }
}

// TODO: return str slice
impl Value<String> for Input {
    fn value(&self) -> String {
        self.value.clone()
    }

    fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_to_end();
    }
}
