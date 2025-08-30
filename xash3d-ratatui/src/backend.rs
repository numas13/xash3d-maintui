use core::ffi::c_int;

use alloc::vec::Vec;
use ratatui::{
    buffer::Cell,
    layout::{Position, Size},
    prelude::*,
};
use xash3d_ui::{color::RGBA, engine, globals, picture::Picture, raw::wrect_s};

use crate::font::{Font, FontMap};

const DEFAULT_FONT_SIZE: u16 = 21;

const XASH_LOGO: &[u8] = include_bytes!("../data/xash_logo.png");

fn scale_font_size(size: u16, width: c_int, _height: c_int) -> u16 {
    let scale = if width > 1920 {
        2.0
    } else if width >= 720 {
        1.375
    } else {
        return size;
    };
    (size as f32 * scale) as u16
}

struct DrawCell {
    index: u16,
    x: i16,
    y: i16,
    fg: RGBA,
}

pub struct XashBackend {
    /// screen width in pixels
    width: c_int,
    /// screen height in pixels
    height: c_int,
    mouse_pos: Position,
    cursor: Position,
    font_map: FontMap,
    // temporary buffer for sorted list of cells optimized for rendering
    cells: Vec<DrawCell>,
    bg: Picture,
}

impl Default for XashBackend {
    fn default() -> Self {
        let globals = globals();
        let width = globals.scrWidth;
        let height = globals.scrHeight;
        let font_size = scale_font_size(DEFAULT_FONT_SIZE, width, height);
        Self {
            width,
            height,
            mouse_pos: Position::ORIGIN,
            cursor: Position::ORIGIN,
            font_map: FontMap::new(Font::new(font_size as isize)),
            cells: Vec::new(),
            bg: Picture::create(c"#mainui/backend/xash_logo.png", XASH_LOGO).unwrap(),
        }
    }
}

impl XashBackend {
    pub fn cell_size_in_pixels(&self) -> Size {
        Size::from(self.font_map.glyph_size())
    }

    pub fn size(&self) -> Size {
        let cell = self.cell_size_in_pixels();
        Size::new(
            self.width as u16 / cell.width,
            self.height as u16 / cell.height,
        )
    }

    pub fn area(&self) -> Rect {
        Rect::from((Position::ORIGIN, self.size()))
    }

    pub fn cursor_position_in_pixels(&self) -> Position {
        self.mouse_pos
    }

    pub fn cursor_position(&self) -> Position {
        self.cursor
    }

    pub fn mouse_to_cursor(&self, mouse: Position) -> Position {
        let cell = self.cell_size_in_pixels();
        let mut cursor = Position::ORIGIN;
        if (0..self.width as u16).contains(&mouse.x) {
            cursor.x = mouse.x / cell.width;
        }
        if (0..self.height as u16).contains(&mouse.y) {
            cursor.y = mouse.y / cell.height;
        }
        cursor
    }

    pub fn set_cursor_position(&mut self, mouse: Position) -> bool {
        if self.mouse_pos == mouse {
            return false;
        }
        self.mouse_pos = mouse;
        self.cursor = self.mouse_to_cursor(mouse);
        true
    }

    pub fn area_to_pixels(&self, area: Rect) -> Rect {
        let cell = self.cell_size_in_pixels();
        Rect::new(
            cell.width * area.x,
            cell.height * area.y,
            cell.width * area.width,
            cell.height * area.height,
        )
    }

    pub fn is_cursor_in_area(&self, area: Rect) -> bool {
        area.contains(self.cursor)
    }

    pub fn cursor_to_item(&self, offset: usize, len: usize) -> Option<usize> {
        let row = self.cursor.y as usize;
        if (offset..offset + len).contains(&row) {
            Some(row - offset)
        } else {
            None
        }
    }

    pub fn cursor_to_item_in_area(&self, offset: usize, len: usize, area: Rect) -> Option<usize> {
        if self.is_cursor_in_area(area) {
            let row = (self.cursor.y - area.y) as usize;
            if (offset..offset + len).contains(&row) {
                return Some(row - offset);
            }
        }
        None
    }

    pub fn decrease_font_size(&mut self) {
        self.set_font_size(self.get_font_size() - 1);
    }

    pub fn increase_font_size(&mut self) {
        self.set_font_size(self.get_font_size() + 1);
    }

    pub fn get_font_size(&self) -> u16 {
        self.font_map.font().size() as u16
    }

    pub fn set_font_size(&mut self, size: u16) {
        let size = size.clamp(8, 128) as isize;
        if size != self.font_map.font().size() {
            self.font_map = FontMap::new(Font::new(size));
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width as c_int;
        self.height = height as c_int;
        self.set_font_size(scale_font_size(DEFAULT_FONT_SIZE, self.width, self.height));
    }

    pub fn draw_background(&mut self) {
        let engine = engine();

        // fill screen with default color
        let s = (self.width, self.height);
        engine.fill_rgba((0, 0), s, color_bg(Color::Reset));

        // draw xash logo at the right bottom corner
        let bg_size = self.bg.size();
        let cell = self.cell_size_in_pixels();
        let screen = self.size();
        let x = (screen.width.saturating_sub(1) * cell.width) as c_int - bg_size.width;
        let y = (screen.height.saturating_sub(1) * cell.height) as c_int - bg_size.height;
        self.bg.set();
        engine.pic_draw_trans((x, y), bg_size, None);
    }

    pub(crate) fn draw_buffer(&mut self, buffer: &Buffer) {
        let cell_size = self.cell_size_in_pixels();
        let cell_width = cell_size.width as c_int;
        let cell_height = cell_size.height as c_int;
        let width = self.width;
        let mut x = 0;
        let mut y = 0;
        let iter = buffer.content().iter().enumerate().filter_map(|(i, cell)| {
            // XXX: numas13: do we need multi-width characters?
            let result = (!cell.skip && *cell != Cell::EMPTY).then_some((i as u16, x, y, cell));
            x += cell_width;
            if x + cell_width > width {
                x = 0;
                y += cell_height;
            }
            result
        });

        // draw background and collect non-empty cells
        let engine = engine();
        let ascent = self.font_map.font().ascent() as c_int;
        for (i, x, y, cell) in iter {
            if cell.bg != Color::Reset {
                engine.fill_rgba((x, y), (cell_width, cell_height), color_bg(cell.bg));
            }
            let y = y + ascent;
            let fg = color_fg(cell.fg);
            if cell.modifier.contains(Modifier::UNDERLINED) {
                engine.fill_rgba((x, y + 1), (cell_width, 2), fg);
            }
            if !cell.symbol().trim_start().is_empty() {
                let x = x as i16;
                let y = y as i16;
                self.cells.push(DrawCell { index: i, x, y, fg });
            }
        }

        // sorting by color results in a less state changes for better performance
        self.cells.sort_unstable_by_key(|i| i.fg);

        // draw non-empty cells
        for draw in self.cells.drain(..) {
            // SAFETY: index is from enumerate over buffer.content()
            let cell = unsafe { buffer.content().get_unchecked(draw.index as usize) };
            for c in cell.symbol().chars() {
                let (pic, info) = self.font_map.get(c, cell.modifier);
                pic.set_with_color(draw.fg);
                let gx = draw.x as c_int + info.bearing_x as c_int;
                let gy = draw.y as c_int + info.bearing_y as c_int;
                let gw = info.w as c_int;
                let gh = info.h as c_int;
                let rect = wrect_s {
                    left: info.x as c_int,
                    right: (info.x + info.w) as c_int,
                    top: info.y as c_int,
                    bottom: (info.y + info.h) as c_int,
                };
                engine.pic_draw_trans((gx, gy), (gw, gh), Some(&rect));
            }
        }
    }
}

fn convert_color(color: Color, is_fg: bool) -> RGBA {
    let color = match color {
        Color::Reset if is_fg => 0xf6f6ef,
        Color::Reset => 0x1a1a1a,
        Color::Black => 0x000000,
        Color::Red => 0xf4005f,
        Color::Green => 0x98e024,
        Color::Yellow => 0xfa8419,
        Color::Blue => 0x9d65ff,
        Color::Magenta => 0xa4307f,
        Color::Cyan => 0x58d1eb,
        Color::Gray => 0xc4c5b5,
        Color::DarkGray => 0x625e4c,
        Color::LightRed => 0xa4305f,
        Color::LightGreen => 0x98e024,
        Color::LightYellow => 0xe0d561,
        Color::LightBlue => 0x9d65ff,
        Color::LightMagenta => 0xf4307f,
        Color::LightCyan => 0x58d1eb,
        Color::White => 0xf6f6ef,
        Color::Rgb(r, g, b) => u32::from_be_bytes([0, r, g, b]),
        Color::Indexed(_) => todo!(),
    };
    let [_, r, g, b] = color.to_be_bytes();
    RGBA::rgb(r, g, b)
}

fn color_bg(color: Color) -> RGBA {
    convert_color(color, false)
}

fn color_fg(color: Color) -> RGBA {
    convert_color(color, true)
}
