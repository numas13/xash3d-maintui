use core::ops::{Deref, DerefMut, Index};

use alloc::{string::ToString, vec::Vec};
use compact_str::{CompactString, ToCompactString};
use ratatui::{
    prelude::*,
    widgets::{HighlightSpacing, StatefulWidgetRef},
};
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    strings::strings,
    ui::{Screen, sound, symbols, utils::Scroll},
    widgets::{Scrollbar, SelectResult, WidgetMut},
};

#[derive(Default)]
pub struct ListState {
    state: ratatui::widgets::ListState,
    last: Option<usize>,
}

impl ListState {
    pub fn new() -> Self {
        let state = ratatui::widgets::ListState::default();
        Self { state, last: None }
    }

    pub fn new_first() -> Self {
        let mut ret = Self::new();
        ret.state.select_first();
        ret
    }

    pub fn cursor_to_index(&self, backend: &XashBackend, area: Rect, len: usize) -> Option<usize> {
        let len = len.saturating_sub(self.offset());
        let row = backend.cursor_to_item_in_area(0, len, area)?;
        Some(self.offset() + row)
    }

    pub fn select_first(&mut self) {
        self.state.select_first();
        self.last = None;
    }

    pub fn prev(&mut self) {
        self.state.select_previous();
        if self.last != self.state.selected() {
            self.last = self.state.selected();
            sound::select_prev();
        }
    }

    pub fn next(&mut self) {
        self.state.select_next();
        if self.last != self.state.selected() {
            self.last = self.state.selected();
            sound::select_next();
        }
    }
}

impl Deref for ListState {
    type Target = ratatui::widgets::ListState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for ListState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

pub struct List {
    style: Style,
    highlight_style: Style,
    pub area: Rect,
    pub state: ListState,
    items: Vec<CompactString>,
    bindings: Vec<(Key, CompactString)>,
    list: Option<ratatui::widgets::List<'static>>,
}

impl List {
    pub fn new<T: ToCompactString>(items: impl IntoIterator<Item = T>) -> Self {
        Self {
            style: Style::new(),
            highlight_style: Style::new()
                .add_modifier(Modifier::BOLD)
                .black()
                .on_yellow(),
            area: Rect::ZERO,
            state: ListState::new(),
            items: items.into_iter().map(|i| i.to_compact_string()).collect(),
            bindings: Vec::new(),
            list: None,
        }
    }

    pub fn new_first<T: ToCompactString>(items: impl IntoIterator<Item = T>) -> Self {
        let mut ret = Self::new(items);
        ret.state.select_first();
        ret
    }

    pub fn set_bindings<I, T>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (Key, T)>,
        T: ToCompactString,
    {
        self.bindings.clear();
        let iter = iter.into_iter().map(|(k, i)| (k, i.to_compact_string()));
        self.bindings.extend(iter);
    }

    pub fn match_binding(&self, key: Key) -> Option<usize> {
        if let Some((_, name)) = self.bindings.iter().find(|(k, _)| *k == key) {
            if let Some(i) = self.items.iter().position(|i| i == name) {
                // self.state.select(Some(i));
                return Some(i);
            }
        }
        None
    }

    pub fn empty() -> Self {
        Self::new_first::<&str>([])
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn set_highlight_style(&mut self, style: Style) {
        self.highlight_style = style;
    }

    pub fn cursor_to_item(&self, backend: &XashBackend) -> Option<usize> {
        self.state.cursor_to_index(backend, self.area, self.len())
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.list = None;
    }

    pub fn push(&mut self, item: impl ToCompactString) {
        self.items.push(item.to_compact_string());
    }

    pub fn extend<T: ToCompactString, I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.items
            .extend(iter.into_iter().map(|i| i.to_compact_string()));
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|i| i.as_str())
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|i| i.as_str())
    }

    fn get_binding(&self, item: &str) -> Option<Key> {
        self.bindings
            .iter()
            .find(|(_, i)| i == item)
            .map(|(i, _)| *i)
    }

    fn create_line(&self, item: &str) -> Line<'static> {
        let strings = strings();
        let s = strings.get(item);
        if let Some(Key::Char(c)) = self.get_binding(item) {
            if c.is_ascii_alphanumeric() {
                if let Some((i, _)) = s
                    .char_indices()
                    .find(|(_, i)| i.to_ascii_lowercase() == c as char)
                {
                    let (head, rest) = s.split_at(i);
                    let (mid, tail) = rest.split_at(1);
                    return Line::default().spans([
                        Span::from(head.to_string()),
                        mid.to_string().yellow().underlined(),
                        Span::from(tail.to_string()),
                    ]);
                }
            }
        }
        Line::from(s.to_string())
    }

    fn init_list(&mut self) {
        let items: Vec<_> = self.items.iter().map(|i| self.create_line(i)).collect();
        self.list = ratatui::widgets::List::new(items)
            .style(self.style)
            .highlight_style(self.highlight_style)
            .highlight_symbol(symbols::HIGHLIGHT_SYMBOL)
            .highlight_spacing(HighlightSpacing::Always)
            .into();
    }
}

impl WidgetMut<SelectResult> for List {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _: &Screen) {
        self.area = area;
        if area.height > 4 && (area.height as usize + 1) < self.items.len() {
            // reserve space for scrollbar
            self.area.width = self.area.width.saturating_sub(1);
        }

        if self.list.is_none() {
            self.init_list();
        }

        if let Some(list) = self.list.as_ref() {
            StatefulWidgetRef::render_ref(list, self.area, buf, &mut self.state);
        }

        if area.height > 4 {
            let scrollbar_style = if self.style.bg.is_none() {
                self.style.gray()
            } else {
                self.style
            };
            Scrollbar::new(self.state.offset(), self.items.len(), 0)
                .style(scrollbar_style)
                .thumb_style(scrollbar_style.red())
                .render(area, buf);
        }
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> SelectResult {
        let key = event.key();

        // try bindings first
        if let Some(i) = self.match_binding(key) {
            return SelectResult::Ok(i);
        }

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
                _ => self.state.prev(),
            },
            _ if key.is_next() => match self.state.selected() {
                None => return SelectResult::Down,
                Some(i) if i + 1 >= self.items.len() => return SelectResult::Down,
                _ => self.state.next(),
            },
            _ if key.is_back() => return SelectResult::Cancel,
            Key::PageUp => self.state.scroll_up_by(half),
            Key::PageDown => self.state.scroll_down_by(half),
            Key::Char(b'u') if event.ctrl() => self.state.scroll_up_by(half),
            Key::Char(b'd') if event.ctrl() => self.state.scroll_down_by(half),
            Key::Home => self.state.select_first(),
            Key::End => self.state.select_last(),
            Key::MouseWheelUp(n) => self.state.scroll_up(n),
            Key::MouseWheelDown(n) => self.state.scroll_down(n, self.items.len(), self.area, 0),
            Key::Mouse(k @ (0 | 1)) => {
                if let Some(i) = self.cursor_to_item(backend) {
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
            _ => return SelectResult::None,
        }
        SelectResult::Select(self.state.selected())
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        if let item @ Some(_) = self.cursor_to_item(backend) {
            self.state.select(item);
            true
        } else {
            false
        }
    }
}

impl Index<usize> for List {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}
