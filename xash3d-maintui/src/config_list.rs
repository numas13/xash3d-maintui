mod back_button;
mod backend;
mod button;
mod entry;
mod label;

use std::ffi::CStr;

use compact_str::ToCompactString;
use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::{
        symbols,
        utils::{self, Scroll},
        Control, Screen, State,
    },
    widgets::{ListState, Scrollbar},
};

pub use self::{
    back_button::BackButton, backend::*, button::Button, entry::ConfigEntry, label::Label,
};

pub enum ConfigAction {
    None,
    Grab,
    Cancel,
    Confirm,
    Control(Control),
}

pub trait ConfigItem {
    fn get_hint(&self) -> Option<&str> {
        None
    }

    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen, style: Style);

    #[allow(unused_variables)]
    fn item_render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {}

    fn item_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction;

    #[allow(unused_variables)]
    fn item_key_event_grab(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction {
        ConfigAction::None
    }

    #[allow(unused_variables)]
    fn item_mouse_event(&mut self, backend: &XashBackend) -> bool {
        false
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
enum Focus {
    #[default]
    Main,
    Grab(usize),
}

pub struct ConfigList {
    title: &'static str,
    state: State<Focus>,
    list_state: ListState,
    items: Vec<Box<dyn ConfigItem>>,
    list_area: Rect,
}

impl ConfigList {
    pub fn with_back(title: &'static str) -> Self {
        let mut ret = Self {
            title,
            state: State::default(),
            list_state: ListState::new_first(),
            items: vec![],
            list_area: Rect::ZERO,
        };
        ret.back_button();
        ret
    }

    pub fn add(&mut self, item: impl ConfigItem + 'static) {
        self.items.push(Box::new(item));
    }

    pub fn back_button(&mut self) {
        self.add(BackButton);
    }

    pub fn label(&mut self, label: impl ToCompactString) {
        self.add(Label::new(label));
    }

    pub fn slider(&mut self, label: &str, cvar: &'static CStr) {
        self.add(
            ConfigEntry::slider_default()
                .label(label)
                .build_for_cvar(cvar),
        );
    }

    pub fn checkbox(&mut self, label: &str, cvar: &'static CStr) {
        self.add(ConfigEntry::checkbox().label(label).build_for_cvar(cvar));
    }

    pub fn popup_list<T>(&mut self, label: &'static str, cvar: &'static CStr, list: T)
    where
        T: IntoIterator,
        T::Item: ToCompactString,
    {
        self.add(ConfigEntry::list(label, list).build_for_cvar(cvar));
    }

    // pub fn button<F: FnMut() -> Control + 'static>(&mut self, label: impl ToString, func: F) {
    //     self.add(Button::new(label, func))
    // }

    fn cursor_to_menu_item(&self, backend: &XashBackend) -> Option<usize> {
        self.list_state
            .cursor_to_index(backend, self.list_area, self.items.len())
    }

    fn set_list_area(&mut self, mut area: Rect) {
        if area.height > 4 && (area.height as usize + 1) < self.items.len() {
            area.width = area.width.saturating_sub(1);
        }

        if let Some(mut selected) = self.list_state.selected() {
            if selected >= self.items.len() {
                selected = self.items.len().saturating_sub(1);
                self.list_state.select(Some(selected));
            }
            let height = area.height as usize;
            let start = self.list_state.offset();
            let end = start + height;
            if selected >= end {
                *self.list_state.offset_mut() = selected.saturating_sub(height) + 1;
            } else if start > selected {
                *self.list_state.offset_mut() = selected;
            }
        }

        self.list_area = area;
    }

    fn draw_list(&mut self, mut area: Rect, buf: &mut Buffer, screen: &Screen) {
        let selected = self.list_state.selected().and_then(|i| self.items.get(i));
        let hint = selected.and_then(|i| i.get_hint());
        area = utils::render_hint(area, buf, self.items.len(), hint);
        self.set_list_area(area);

        let layout = Layout::horizontal([Constraint::Length(1), Constraint::Percentage(100)]);
        let items = self
            .items
            .iter_mut()
            .enumerate()
            .skip(self.list_state.offset());

        for ((i, item), item_area) in items.zip(self.list_area.rows()) {
            let [point_area, content_area] = layout.areas(item_area);
            let mut style = Style::default();
            if self.list_state.selected() == Some(i) {
                style = style.add_modifier(Modifier::BOLD).black();
                match self.state.focus() {
                    Focus::Main => style = style.on_yellow(),
                    Focus::Grab(..) => style = style.on_green(),
                };
                Line::raw(symbols::HIGHLIGHT_SYMBOL)
                    .style(style)
                    .render(point_area, buf);
            }

            item.item_render_inline(content_area, buf, screen, style);
        }
    }

    fn draw_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        if area.height > 4 {
            Scrollbar::new(self.list_state.offset(), self.items.len(), 0)
                .thumb_style(Style::new().red())
                .render(area, buf);
        }
    }

    fn draw_popup(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        if let Focus::Grab(i) = self.state.focus() {
            self.items[*i].item_render(area, buf, screen);
        }
    }

    pub fn draw_centered(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let area = utils::centered_rect(80, 30, area);
        self.draw(area, buf, screen);
    }

    pub fn draw(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        let inner_area = utils::main_block(self.title, area, buf);
        self.draw_list(inner_area, buf, screen);
        self.draw_scrollbar(inner_area, buf);
        self.draw_popup(area, buf, screen);
    }

    pub fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> Control {
        let key = event.key();
        match self.state.focus() {
            Focus::Main => match key {
                _ if key.is_exec() => {
                    if let Some(i) = self.list_state.selected() {
                        match self.items[i].item_key_event(backend, event) {
                            ConfigAction::Control(control) => return control,
                            ConfigAction::Grab => self.state.select(Focus::Grab(i)),
                            _ => {}
                        }
                    }
                }
                _ if key.is_prev() => self.list_state.prev(),
                _ if key.is_next() => {
                    if self
                        .list_state
                        .selected()
                        .is_some_and(|i| (i + 1) < self.items.len())
                    {
                        self.list_state.next();
                    }
                }
                _ if key.is_back() => return Control::Back,
                Key::MouseWheelUp(n) => self.list_state.scroll_up(n),
                Key::MouseWheelDown(n) => {
                    self.list_state
                        .scroll_down(n, self.items.len(), self.list_area, 0)
                }
                Key::TouchStart(..) => {
                    if let Some(i) = self.cursor_to_menu_item(backend) {
                        if let ConfigAction::Grab = self.items[i].item_key_event(backend, event) {
                            self.list_state.select(Some(i));
                            self.state.select(Focus::Grab(i));
                        }
                    }
                }
                Key::Mouse(0) => {
                    if let Some(i) = self.cursor_to_menu_item(backend) {
                        self.list_state.select(Some(i));
                        match self.items[i].item_key_event(backend, event) {
                            ConfigAction::Control(control) => return control,
                            ConfigAction::Grab => self.state.select(Focus::Grab(i)),
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            Focus::Grab(i) => match self.items[*i].item_key_event_grab(backend, event) {
                ConfigAction::Control(control) => return control,
                ConfigAction::Confirm => self.state.confirm_default(),
                ConfigAction::Cancel => self.state.cancel_default(),
                _ => {}
            },
        }
        Control::None
    }

    pub fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        match self.state.focus() {
            Focus::Main => {
                if let item @ Some(_) = self.cursor_to_menu_item(backend) {
                    self.list_state.select(item);
                    true
                } else {
                    false
                }
            }
            Focus::Grab(i) => self.items[*i].item_mouse_event(backend),
        }
    }
}
