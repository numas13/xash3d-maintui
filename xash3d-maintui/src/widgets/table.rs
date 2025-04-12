use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use ratatui::{
    prelude::*,
    widgets::{HighlightSpacing, Row, Table, TableState},
};
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{
        symbols,
        utils::{self, Scroll},
    },
};

use super::SelectResult;

pub struct MyTable<T> {
    pub area: Rect,
    pub header_areas: Rc<[Rect]>,
    pub state: TableState,
    pub items: Vec<T>,
}

impl<T> MyTable<T> {
    pub fn new() -> Self {
        Self::with_state(TableState::new())
    }

    pub fn new_first() -> Self {
        let mut state = TableState::new();
        state.select(Some(0));
        Self::with_state(state)
    }

    pub fn with_state(state: TableState) -> Self {
        Self {
            area: Rect::ZERO,
            header_areas: Rc::new([]),
            state,
            items: Default::default(),
        }
    }

    pub fn create_table<'a>(
        &mut self,
        mut area: Rect,
        header: Row<'a>,
        widths: &[Constraint],
    ) -> Table<'a> {
        area.x += 1;
        area.width = area.width.saturating_sub(1);
        area.height = 1;
        self.header_areas = Layout::horizontal(widths).spacing(1).split(area);
        Table::default().header(header).widths(widths)
    }

    pub fn cursor_to_header_column(&self, position: Position) -> Option<usize> {
        self.header_areas.iter().position(|i| i.contains(position))
    }

    pub fn cursor_to_table_item(&self, backend: &XashBackend) -> Option<usize> {
        // FIXME: optional header
        let offset = self.state.offset();
        let len = self.items.len().saturating_sub(offset);
        backend
            .cursor_to_item_in_area(1, len, self.area)
            .map(|i| i + offset)
    }

    pub fn draw(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        table: ratatui::widgets::Table,
        focused: bool,
        f: impl FnMut(&T) -> Option<Row>,
    ) {
        let style = if focused {
            Style::new()
                .add_modifier(Modifier::BOLD)
                .black()
                .on_yellow()
        } else {
            Style::new()
        };
        let rows: Vec<_> = self.items.iter().filter_map(f).collect();
        let table = table
            .rows(rows)
            .column_spacing(1)
            .style(Style::new().white())
            .row_highlight_style(style)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(symbols::HIGHLIGHT_SYMBOL);

        self.area = area;
        // reserve space for scrollbar
        self.area.width = self.area.width.saturating_sub(1);
        StatefulWidget::render(table, self.area, buf, &mut self.state);

        if area.height > 4 {
            // FIXME: optional header
            utils::render_scrollbar(buf, area, self.items.len(), self.state.offset(), 1);
        }
    }

    pub fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> SelectResult {
        let key = event.key();
        let half = self.area.height / 2;
        match key {
            _ if key.is_exec() => {
                if let Some(i) = self.state.selected() {
                    return SelectResult::Ok(i);
                } else {
                    return SelectResult::None;
                }
            }
            _ if key.is_prev() => match self.state.selected() {
                None | Some(0) => return SelectResult::Up,
                _ => self.state.select_previous(),
            },
            _ if key.is_next() => match self.state.selected() {
                None => return SelectResult::Down,
                Some(i) if i + 1 >= self.items.len() => return SelectResult::Down,
                _ => self.state.select_next(),
            },
            _ if key.is_back() => return SelectResult::Cancel,
            Key::PageUp => self.state.scroll_up_by(half),
            Key::PageDown => self.state.scroll_down_by(half),
            Key::Char(b'u') if event.ctrl() => self.state.scroll_up_by(half),
            Key::Char(b'd') if event.ctrl() => self.state.scroll_down_by(half),
            Key::Home => self.state.select_first(),
            Key::End => self.state.select_last(),
            Key::Mouse(k @ (0 | 1)) => {
                if let Some(i) = self.cursor_to_table_item(backend) {
                    self.state.select(Some(i));
                    if k == 0 {
                        return SelectResult::Ok(i);
                    } else {
                        return SelectResult::ContextMenu(i);
                    }
                } else {
                    return SelectResult::None;
                }
            }
            Key::MouseWheelUp(n) => self.state.scroll_up(n),
            Key::MouseWheelDown(n) => self.state.scroll_down(n, self.items.len(), self.area, 1),
            _ => return SelectResult::None,
        }
        SelectResult::Select(self.state.selected())
    }

    pub fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        if let Some(i) = self.cursor_to_table_item(backend) {
            self.state.select(Some(i));
            true
        } else {
            false
        }
    }
}

impl<T> Default for MyTable<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Deref for MyTable<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for MyTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
