use std::{cmp, ffi::CStr};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, ListState, Paragraph, TableState, Wrap},
};
use unicode_width::UnicodeWidthStr;
use xash3d_ui::{engine, engine::CVar};

use crate::strings;

pub fn is_wide(area: Rect) -> bool {
    area.width >= 80
}

pub fn main_block_border_style() -> Style {
    Style::new().yellow()
}

pub fn main_block(title: &str, area: Rect, buf: &mut Buffer) -> Rect {
    let block = Block::default()
        .title(strings::get(title).yellow())
        .borders(Borders::ALL)
        .border_style(main_block_border_style());
    let inner_area = block.inner(area);
    block.render(area, buf);
    inner_area
}

pub fn popup_block_style() -> Style {
    Style::default().black().on_gray()
}

pub fn popup_block(title: &str) -> Block {
    Block::default()
        .title(strings::get(title).black())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(popup_block_style())
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let area = if area.width > width {
        Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .areas::<3>(area)[1]
    } else {
        area
    };
    if area.height > height {
        Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .areas::<3>(area)[1]
    } else {
        area
    }
}

pub fn menu_block(title: &str, area: Rect, buf: &mut Buffer) -> Rect {
    main_block(title, centered_rect(30, 20, area), buf)
}

// pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
//     // Cut the given rectangle into three vertical pieces
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints([
//             Constraint::Percentage((100 - percent_y) / 2),
//             Constraint::Percentage(percent_y),
//             Constraint::Percentage((100 - percent_y) / 2),
//         ])
//         .split(r);
//
//     // Then cut the middle vertical piece into three width-wise pieces
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([
//             Constraint::Percentage((100 - percent_x) / 2),
//             Constraint::Percentage(percent_x),
//             Constraint::Percentage((100 - percent_x) / 2),
//         ])
//         .split(popup_layout[1])[1] // Return the middle chunk
// }

pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let height = cmp::min(height, cmp::max(r.height.saturating_sub(4), 16));
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(r);

    let width = cmp::min(width, cmp::max(r.width.saturating_sub(4), 24));
    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn render_scrollbar(buf: &mut Buffer, area: Rect, len: usize, offset: usize, extra: usize) {
    crate::widgets::Scrollbar::new(offset, len, extra).render(area, buf);
}

pub trait Scroll {
    fn offset_mut(&mut self) -> &mut usize;

    fn scroll_up(&mut self, n: u16) {
        let offset = self.offset_mut();
        *offset = offset.saturating_sub(n as usize);
    }

    fn scroll_down(&mut self, n: u16, len: usize, area: Rect, extra: usize) {
        let offset = self.offset_mut();
        let max = len.saturating_sub((area.height as usize).saturating_sub(extra));
        *offset = cmp::min(*offset + n as usize, max);
    }
}

impl Scroll for ListState {
    fn offset_mut(&mut self) -> &mut usize {
        ListState::offset_mut(self)
    }
}

impl Scroll for TableState {
    fn offset_mut(&mut self) -> &mut usize {
        TableState::offset_mut(self)
    }
}

fn count_lines(s: &str, width: u16) -> usize {
    let mut lines = 1;
    let mut w = 0;
    for i in s.split(' ') {
        let x = i.width();
        if w + x > width as usize {
            lines += 1;
            w = 0;
        }
        w += x + 1;
    }
    lines
}

pub fn render_hint(area: Rect, buf: &mut Buffer, items: usize, hint: Option<&str>) -> Rect {
    if let Some(hint) = hint {
        let menu_height = items as u16;
        let hint_height = count_lines(hint, area.width) as u16;
        if area.height > (menu_height + hint_height) {
            let [area, hint_area] =
                Layout::vertical([Constraint::Percentage(100), Constraint::Min(hint_height)])
                    .areas(area);
            Paragraph::new(hint.gray())
                .wrap(Wrap { trim: true })
                .render(hint_area, buf);
            return area;
        }
    }
    area
}

pub fn cvar_read<T: CVar>(name: &CStr) -> T {
    engine().cvar(name)
}

pub fn cvar_write<T: CVar>(name: &CStr, value: T) {
    engine().cvar_set(name, value);
}

pub fn is_dev() -> bool {
    engine().cvar::<f32>(c"developer") as i32 > 0
}
