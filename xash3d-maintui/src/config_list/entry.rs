use core::ffi::CStr;

use alloc::boxed::Box;
use compact_str::{CompactString, ToCompactString};
use ratatui::prelude::*;
use unicode_width::UnicodeWidthStr;
use xash3d_ratatui::XashBackend;
use xash3d_ui::CVar;

use crate::{
    input::{Key, KeyEvent},
    ui::{sound, Screen},
    widgets::{Checkbox, ConfirmResult, Input, ListPopup, SelectResult, Slider, Value, WidgetMut},
};

use super::{CVarBackend, ConfigAction, ConfigBackend, ConfigItem};

const LABEL_WIDTH: u16 = 32;

pub struct ConfigEntryBuilder<T> {
    label: Option<CompactString>,
    hint: Option<CompactString>,
    note: Option<CompactString>,
    widget: T,
    fixed_value: Option<CompactString>,
}

impl<T> ConfigEntryBuilder<T> {
    pub fn new(widget: T) -> Self {
        Self {
            label: None,
            hint: None,
            note: None,
            widget,
            fixed_value: None,
        }
    }

    pub fn label(mut self, label: impl ToCompactString) -> Self {
        self.label = Some(label.to_compact_string());
        self
    }

    pub fn note(mut self, note: impl ToCompactString) -> Self {
        self.note = Some(note.to_compact_string());
        self
    }

    pub fn hint(mut self, hint: impl ToCompactString) -> Self {
        self.hint = Some(hint.to_compact_string());
        self
    }

    pub fn fixed_value(mut self, value: impl ToCompactString) -> Self {
        self.fixed_value = Some(value.to_compact_string());
        self
    }

    pub fn build<V, B>(self, backend: B) -> ConfigEntry<V, T>
    where
        T: Value<V>,
        B: ConfigBackend<V> + 'static,
    {
        let value = backend.read();
        let mut ret = ConfigEntry {
            label: self.label,
            hint: self.hint,
            widget: self.widget,
            backend: Box::new(backend),
            note: self.note,
            fixed_value: self.fixed_value,
        };
        if let Some(value) = value {
            ret.widget.set_value(value);
        }
        ret
    }

    pub fn build_for_cvar<'a, V>(self, name: &'static CStr) -> ConfigEntry<V, T>
    where
        V: CVar<'a> + 'static,
        T: Value<V>,
    {
        self.build(CVarBackend::new(name))
    }
}

pub struct ConfigEntry<V, T> {
    label: Option<CompactString>,
    hint: Option<CompactString>,
    note: Option<CompactString>,
    widget: T,
    backend: Box<dyn ConfigBackend<V>>,
    fixed_value: Option<CompactString>,
}

impl<T> ConfigEntry<(), T> {
    pub fn builder(widget: T) -> ConfigEntryBuilder<T> {
        ConfigEntryBuilder::new(widget)
    }
}

impl ConfigEntry<(), ()> {
    pub fn checkbox() -> ConfigEntryBuilder<Checkbox> {
        ConfigEntryBuilder::new(Checkbox::new())
    }

    pub fn slider(min: f32, max: f32, step: f32) -> ConfigEntryBuilder<Slider> {
        ConfigEntryBuilder::new(Slider::builder().min(min).max(max).step(step).build())
    }

    pub fn slider_step(step: f32) -> ConfigEntryBuilder<Slider> {
        Self::slider(0.0, 1.0, step)
    }

    pub fn slider_default() -> ConfigEntryBuilder<Slider> {
        ConfigEntryBuilder::new(Slider::new())
    }

    pub fn input() -> ConfigEntryBuilder<Input> {
        ConfigEntryBuilder::new(Input::new())
    }

    pub fn list<T>(title: impl ToCompactString, items: T) -> ConfigEntryBuilder<ListPopup>
    where
        T: IntoIterator,
        T::Item: ToCompactString,
    {
        ConfigEntryBuilder::new(ListPopup::new(title, items))
    }
}

impl<V, T> ConfigEntry<V, T> {
    pub fn inner(&self) -> &T {
        &self.widget
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.widget
    }

    pub fn is_enabled(&self) -> bool {
        self.backend.is_enabled()
    }

    fn render_label_impl(&self, area: Rect, buf: &mut Buffer, label: &str, style: Style) {
        let width = area.width.saturating_sub(1) as usize;
        let line = match label.char_indices().nth(width) {
            Some((i, _)) => Line::from_iter([
                Span::from(&label[..i]),
                Span::from(">").style(Style::new().dark_gray()),
            ]),
            None => Line::raw(label),
        };
        line.style(style).render(area, buf);
    }

    fn render_label(&self, area: Rect, buf: &mut Buffer, label: &str, style: Style) -> Rect {
        let layout = if area.width >= LABEL_WIDTH * 2 {
            Layout::horizontal([Constraint::Length(LABEL_WIDTH), Constraint::Percentage(100)])
        } else {
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(100)])
        };
        let [label_area, widget_area] = layout.areas(area);
        self.render_label_impl(label_area, buf, label, style);
        widget_area
    }

    fn render_default_label(&self, area: Rect, buf: &mut Buffer, style: Style) -> Rect {
        let label = self.label.as_ref().map_or("TODO", |i| i.as_str());
        self.render_label(area, buf, label, style)
    }
}

impl<V, T> ConfigEntry<V, T>
where
    V: Copy,
    T: Value<V>,
{
    pub fn set(&mut self, value: V) {
        if self.is_enabled() {
            self.widget.set_value(value);
            self.backend.write(value);
        }
    }
}

impl<'a, V, T> WidgetMut<ConfirmResult> for ConfigEntry<V, T>
where
    V: CVar<'a> + Copy,
    T: WidgetMut<ConfirmResult> + Value<V>,
{
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.widget.render(area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        let action = self.widget.key_event(backend, event);
        if action == ConfirmResult::Ok || matches!(key, Key::Touch(..)) {
            self.set(self.widget.value());
        }
        action
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.widget.mouse_event(backend)
    }
}

impl<T> WidgetMut<SelectResult> for ConfigEntry<usize, T>
where
    T: WidgetMut<SelectResult> + Value<usize>,
{
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        self.widget.render(area, buf, screen);
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> SelectResult {
        let action = self.widget.key_event(backend, event);
        if let SelectResult::Ok(i) = action {
            self.set(i);
        }
        action
    }

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.widget.mouse_event(backend)
    }
}

impl ConfigEntry<bool, Checkbox> {
    pub fn toggle(&mut self) {
        if self.is_enabled() {
            self.widget.set_value(!self.widget.value());
            self.set(self.widget.value());
            sound::select_changed();
        }
    }
}

impl ConfigItem for ConfigEntry<bool, Checkbox> {
    fn get_hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen, style: Style) {
        let area = self.render_default_label(area, buf, style);
        let style = if self.is_enabled() {
            style
        } else {
            style.dark_gray()
        };
        self.inner_mut().set_style(style);
        self.render(area, buf, screen);
    }

    fn item_key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfigAction {
        let key = event.key();
        if key.is_exec() || matches!(key, Key::Mouse(0)) {
            self.toggle();
        }
        ConfigAction::None
    }

    fn item_mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.mouse_event(backend)
    }
}

impl ConfigItem for ConfigEntry<f32, Slider> {
    fn get_hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn item_render_inline(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        screen: &Screen,
        mut style: Style,
    ) {
        if !self.is_enabled() {
            style = style.dark_gray();
        }
        let area = self.render_default_label(area, buf, style);
        self.inner_mut().set_style(style);
        self.render(area, buf, screen);
    }

    fn item_key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction {
        if !self.is_enabled() {
            return ConfigAction::None;
        }
        let key = event.key();
        if let Key::TouchStop(..) = key {
            return ConfigAction::Grab;
        }
        match key {
            Key::Mouse(0) => {
                self.key_event(backend, event);
            }
            Key::TouchStart(cursor) => {
                if self.inner().is_gauge_area(cursor) {
                    self.key_event(backend, event);
                    return ConfigAction::Grab;
                }
            }
            _ => return ConfigAction::Grab,
        }
        ConfigAction::None
    }

    fn item_key_event_grab(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction {
        let key = event.key();
        match self.key_event(backend, event) {
            ConfirmResult::None => {}
            ConfirmResult::Cancel => return ConfigAction::Cancel,
            ConfirmResult::Ok => {
                if let Key::TouchStop(..) = key {
                    return ConfigAction::Confirm;
                }
            }
        }
        ConfigAction::None
    }

    fn item_mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.mouse_event(backend)
    }
}

impl ConfigEntry<usize, ListPopup> {
    pub fn get_index(&self) -> usize {
        self.backend.read().unwrap_or_else(|| self.widget.value())
    }
}

impl ConfigItem for ConfigEntry<usize, ListPopup> {
    fn get_hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn item_render_inline(&mut self, area: Rect, buf: &mut Buffer, _: &Screen, mut style: Style) {
        if !self.is_enabled() {
            style = style.dark_gray();
        }
        let label = self.label.as_deref().unwrap_or(self.inner().title());
        let area = self.render_label(area, buf, label, style);
        let value = match &self.fixed_value {
            Some(value) => value,
            None => self
                .inner()
                .get(self.get_index())
                .unwrap_or("Invalid")
                .trim_end(),
        };
        Line::raw(value).style(style).render(area, buf);

        if let Some(note) = &self.note {
            let width = note.width() as u16;
            if value.len() as u16 + width < area.width {
                let note_area = Rect {
                    x: area.right() - width,
                    width,
                    ..area
                };
                Line::raw(note.as_str())
                    .style(style.dark_gray())
                    .render(note_area, buf);
            }
        }
    }

    fn item_render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen) {
        if self.inner().state.selected().is_none() {
            let index = self.get_index();
            self.inner_mut().state.select(Some(index));
        }
        self.render(area, buf, screen);
    }

    fn item_key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfigAction {
        if self.is_enabled() {
            let key = event.key();
            if key.is_exec() || matches!(key, Key::Mouse(0)) {
                return ConfigAction::Grab;
            }
        }
        ConfigAction::None
    }

    fn item_key_event_grab(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction {
        match self.key_event(backend, event) {
            SelectResult::Ok(_) => return ConfigAction::Confirm,
            SelectResult::Cancel => return ConfigAction::Cancel,
            _ => {}
        }
        ConfigAction::None
    }

    fn item_mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.mouse_event(backend)
    }
}

impl ConfigItem for ConfigEntry<CompactString, Input> {
    fn get_hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn item_render_inline(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        screen: &Screen,
        mut style: Style,
    ) {
        if !self.is_enabled() {
            style = style.dark_gray();
        }
        let area = self.render_default_label(area, buf, style);
        self.widget.set_style(style);
        self.widget.render(area, buf, screen);
    }

    fn item_key_event(&mut self, _: &XashBackend, event: KeyEvent) -> ConfigAction {
        if self.is_enabled() {
            let key = event.key();
            if key.is_exec() || matches!(key, Key::Mouse(0)) {
                self.widget.show_cursor(true);
                return ConfigAction::Grab;
            }
        }
        ConfigAction::None
    }

    fn item_key_event_grab(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfigAction {
        match self.widget.key_event(backend, event) {
            ConfirmResult::Ok => {
                self.backend.write(self.widget.value().into());
                self.widget.show_cursor(false);
                ConfigAction::Confirm
            }
            _ => ConfigAction::None,
        }
    }

    fn item_mouse_event(&mut self, backend: &XashBackend) -> bool {
        self.widget.mouse_event(backend)
    }
}
