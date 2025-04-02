mod button;
mod checkbox;
mod image;
mod input;
mod list;
mod list_popup;
mod password_popup;
mod popup;
mod scrollbar;
mod slider;
mod table;

use ratatui::prelude::*;
use xash3d_ratatui::XashBackend;

pub use self::button::Button;
pub use self::checkbox::Checkbox;
pub use self::image::Image;
pub use self::input::Input;
pub use self::list::{List, ListState};
pub use self::list_popup::ListPopup;
pub use self::password_popup::PasswordPopup;
pub use self::popup::ConfirmPopup;
pub use self::scrollbar::Scrollbar;
pub use self::slider::Slider;
pub use self::table::{MyTable, TableHeader};

use crate::{
    input::KeyEvent,
    ui::{Control, Screen},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ConfirmResult {
    None,
    Cancel,
    Ok,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputResult<T = String> {
    None,
    Cancel,
    Ok(T),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SelectResult {
    None,
    Cancel,
    Up,
    Down,
    Select(Option<usize>),
    Ok(usize),
    ContextMenu(usize),
}

impl SelectResult {
    pub fn to_control<F: FnMut(usize) -> Control>(self, mut f: F) -> Control {
        match self {
            SelectResult::Ok(i) => f(i),
            SelectResult::Cancel => Control::Back,
            _ => Control::None,
        }
    }
}

#[allow(unused_variables)]
pub trait WidgetMut<T> {
    fn render(&mut self, area: Rect, buf: &mut Buffer, screen: &Screen);

    fn key_event(&mut self, backend: &XashBackend, event: KeyEvent) -> T;

    fn mouse_event(&mut self, backend: &XashBackend) -> bool {
        false
    }
}

pub trait Value<T = f32> {
    fn value(&self) -> T;

    fn set_value(&mut self, value: T);
}
