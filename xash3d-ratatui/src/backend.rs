use alloc::vec::Vec;
use ratatui::{
    buffer::Cell,
    layout::{Position, Size},
    prelude::*,
};
use xash3d_ui::{
    color::RGBA,
    misc::{Point as UiPoint, Rect as UiRect, Size as UiSize},
    picture::Picture,
    prelude::*,
};

use crate::font::{Font, FontMap};

const DEFAULT_FONT_SIZE: u16 = 21;

const XASH_LOGO: &[u8] = include_bytes!("../data/xash_logo.png");

fn scale_font_size(size: u16, width: u32, _height: u32) -> u16 {
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
    engine: UiEngineRef,
    /// screen width in pixels
    width: u32,
    /// screen height in pixels
    height: u32,
    mouse_pos: Position,
    cursor: Position,
    font_map: FontMap,
    // temporary buffer for sorted list of cells optimized for rendering
    cells: Vec<DrawCell>,
    bg: Picture,
}

impl XashBackend {
    pub fn new(engine: UiEngineRef) -> Self {
        let globals = &engine.globals;
        let width = globals.screen_width();
        let height = globals.screen_height();
        let font_size = scale_font_size(DEFAULT_FONT_SIZE, width, height);
        Self {
            engine,
            width,
            height,
            mouse_pos: Position::ORIGIN,
            cursor: Position::ORIGIN,
            font_map: FontMap::new(engine, Font::new(font_size as isize)),
            cells: Vec::new(),
            bg: engine
                .pic_create(c"#mainui/backend/xash_logo.png", XASH_LOGO)
                .unwrap(),
        }
    }

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
            self.font_map = FontMap::new(self.engine, Font::new(size));
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.set_font_size(scale_font_size(DEFAULT_FONT_SIZE, self.width, self.height));
    }

    pub fn draw_background(&mut self) {
        // fill screen with default color
        let size = UiSize::new(self.width, self.height);
        self.engine.fill_rgba(color_bg(Color::Reset), size.into());

        // draw xash logo at the right bottom corner
        let size = self.bg.size();
        let cell = self.cell_size_in_pixels();
        let screen = self.size();
        let x = (screen.width.saturating_sub(1) * cell.width) as i32 - size.width as i32;
        let y = (screen.height.saturating_sub(1) * cell.height) as i32 - size.height as i32;
        let area = size.to_rect(UiPoint::new(x, y));
        self.bg.draw_trans(RGBA::WHITE, area, None);
    }

    pub(crate) fn draw_buffer(&mut self, buffer: &Buffer) {
        let cell_size = self.cell_size_in_pixels();
        let cell_width = cell_size.width as i32;
        let cell_height = cell_size.height as i32;
        let width = self.width;
        let mut x = 0;
        let mut y = 0;
        let iter = buffer.content().iter().enumerate().filter_map(|(i, cell)| {
            // XXX: numas13: do we need multi-width characters?
            let result = (!cell.skip && *cell != Cell::EMPTY).then_some((i as u16, x, y, cell));
            x += cell_width;
            if x + cell_width > width as i32 {
                x = 0;
                y += cell_height;
            }
            result
        });

        // draw background and collect non-empty cells
        let ascent = self.font_map.font().ascent() as i32;
        for (i, x, y, cell) in iter {
            if cell.bg != Color::Reset {
                let area = UiRect::new(x, y, cell_width as u32, cell_height as u32);
                self.engine.fill_rgba(color_bg(cell.bg), area);
            }
            let y = y + ascent;
            let fg = color_fg(cell.fg);
            if cell.modifier.contains(Modifier::UNDERLINED) {
                let area = UiRect::new(x, y + 1, cell_width as u32, 2);
                self.engine.fill_rgba(fg, area);
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
                let area = UiRect::new(
                    draw.x as i32 + info.bearing_x as i32,
                    draw.y as i32 + info.bearing_y as i32,
                    info.w as u32,
                    info.h as u32,
                );
                let pic_area =
                    UiRect::new(info.x as i32, info.y as i32, info.w as u32, info.h as u32);
                pic.draw_trans(draw.fg, area, Some(pic_area));
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
