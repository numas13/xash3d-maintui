use ratatui::{prelude::*, widgets::Gauge};
use xash3d_ratatui::XashBackend;

use crate::{
    input::{Key, KeyEvent},
    ui::Screen,
    widgets::{ConfirmResult, Value, WidgetMut},
};

pub struct SliderBuilder(Slider);

impl SliderBuilder {
    pub fn step(mut self, step: f32) -> Self {
        self.0.step = step;
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.0.max = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.0.max = max;
        self
    }

    pub fn build(mut self) -> Slider {
        self.0.set_value(self.0.value());
        self.0
    }
}

pub struct Slider {
    value: f32,
    step: f32,
    min: f32,
    max: f32,
    style: Style,
    area: Rect,
}

impl Default for Slider {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Slider {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> SliderBuilder {
        SliderBuilder(Self {
            value: 0.0,
            step: 0.01,
            min: 0.0,
            max: 1.0,
            style: Style::default(),
            area: Rect::ZERO,
        })
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(self.min, self.max);
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn ratio(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.value = ratio.clamp(0.0, 1.0) * (self.max - self.min) + self.min;
    }

    pub fn is_gauge_area(&self, cursor: Position) -> bool {
        self.area.contains(cursor)
    }

    fn cursor_to_ratio_touch(&self, backend: &XashBackend) -> f32 {
        let pos = backend.cursor_position_in_pixels();
        let area = backend.area_to_pixels(self.area);
        let x = pos.x.clamp(area.left(), area.right());
        (x - area.x) as f32 / area.width as f32
    }

    fn cursor_to_ratio(&self, backend: &XashBackend) -> Option<f32> {
        let pos = backend.cursor_position_in_pixels();
        let area = backend.area_to_pixels(self.area);
        if area.contains(pos) {
            Some((pos.x - area.x) as f32 / area.width as f32)
        } else {
            None
        }
    }
}

impl WidgetMut<ConfirmResult> for Slider {
    fn render(&mut self, area: Rect, buf: &mut Buffer, _: &Screen) {
        Gauge::default()
            .gauge_style(self.style)
            .ratio(self.ratio() as f64)
            .render(area, buf);
        self.area = area;
    }

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> ConfirmResult {
        let key = event.key();
        match key {
            Key::Char(b'e') | Key::Enter => ConfirmResult::Cancel,
            Key::Char(b'h') | Key::ArrowLeft => {
                self.set_value(self.value() - self.step());
                ConfirmResult::Ok
            }
            Key::Char(b'l') | Key::ArrowRight => {
                self.set_value(self.value() + self.step());
                ConfirmResult::Ok
            }
            Key::Char(b'q') | Key::Escape => ConfirmResult::Cancel,
            Key::Mouse(0) | Key::MouseWheelLeft(_) | Key::MouseWheelRight(_) => {
                if let Some(ratio) = self.cursor_to_ratio(backend) {
                    self.set_ratio(ratio);
                    ConfirmResult::Ok
                } else {
                    ConfirmResult::None
                }
            }
            Key::Touch(..) => {
                let ratio = self.cursor_to_ratio_touch(backend);
                self.set_ratio(ratio);
                ConfirmResult::None
            }
            Key::TouchStop(..) => ConfirmResult::Ok,
            _ => ConfirmResult::None,
        }
    }
}

impl Value<f32> for Slider {
    fn value(&self) -> f32 {
        self.value
    }

    fn set_value(&mut self, value: f32) {
        Self::set_value(self, value);
    }
}
