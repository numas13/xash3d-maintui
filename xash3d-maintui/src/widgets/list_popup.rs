use std::{
    cmp,
    ops::{Deref, DerefMut},
};

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::KeyEvent,
    ui::{utils, Screen},
    widgets::{List, SelectResult, Value, WidgetMut},
};

pub struct ListPopup {
    title: String,
    list: List,
    width: u16,
}

impl ListPopup {
    pub fn new<T>(title: impl ToString, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: ToString,
    {
        let title = title.to_string();

        let mut list = List::new_first(items);
        list.set_style(Style::default().black().on_gray());
        list.set_highlight_style(Style::new().add_modifier(Modifier::BOLD).white().on_black());
        list.set_popup(true);

        let content_width = list
            .iter()
            .map(|i| i.trim().chars().count())
            .max()
            .unwrap_or(0);

        let width = cmp::max(content_width, title.chars().count()) as u16 + 4;

        Self { title, list, width }
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl WidgetMut<SelectResult> for ListPopup {
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let width = self.width;
        let height = 2 + self.list.len() as u16;
        let area = utils::centered_rect_fixed(width, height, area);

        let block = utils::popup_block(&self.title);
        let list_area = block.inner(area);
        block.render(area, buf);

        self.list.render(list_area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> SelectResult {
        self.list.key_event(backend, event)
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.list.mouse_event(backend)
    }
}

impl Value<usize> for ListPopup {
    fn value(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    fn set_value(&mut self, value: usize) {
        self.state.select(Some(value));
    }
}

impl Deref for ListPopup {
    type Target = List;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for ListPopup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
