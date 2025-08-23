use core::ffi::c_int;

use alloc::vec::Vec;
use ratatui::{
    buffer::Cell,
    layout::{Position, Size},
    prelude::*,
};
use xash3d_ui::{color::RGBA, engine, globals, picture::Picture, raw::wrect_s};

use crate::{
    bmp::Bmp,
    font::{Font, FontMap},
};

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

pub struct XashBackend {
    width: c_int,
    height: c_int,
    align: Position,
    mouse_pos: Position,
    cursor: Position,
    font_map: FontMap,
    // temporary buffer for sorted list of cells optimized for rendering
    cells: Vec<(u16, u16, RGBA, *const Cell)>,
    bg: Picture,
}

impl Default for XashBackend {
    fn default() -> Self {
        let mut pixel = Bmp::builder(1, 1).build();
        pixel.set_pixel(0, 0, 255, 255, 255, 255);

        let globals = globals();
        let width = globals.scrWidth;
        let height = globals.scrHeight;
        let font_size = scale_font_size(DEFAULT_FONT_SIZE, width, height);

        let bg = Picture::create(c"#mainui/backend/xash_logo.png", XASH_LOGO);

        let mut ret = Self {
            width,
            height,
            align: Position::ORIGIN,
            mouse_pos: Position::ORIGIN,
            cursor: Position::ORIGIN,
            font_map: FontMap::new(Font::new(font_size as isize)),
            cells: Vec::new(),
            bg,
        };
        ret.calc_alignment();
        ret
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

    fn calc_alignment(&mut self) {
        // let (w, h) = self.get_cell_size();
        // self.align.x = self.width as u16 % w / 2;
        // self.align.y = self.height as u16 % h / 2;
    }

    pub fn alignment_offset(&self) -> Position {
        self.align
    }

    pub fn mouse_to_cursor(&self, mouse: Position) -> Position {
        let cell = self.cell_size_in_pixels();
        let align = self.alignment_offset();
        let mut cursor = Position::ORIGIN;
        if (align.x..self.width as u16 - align.x).contains(&mouse.x) {
            cursor.x = (mouse.x - align.x) / cell.width;
        }
        if (align.y..self.height as u16 - align.y).contains(&mouse.y) {
            cursor.y = (mouse.y - align.y) / cell.height;
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
        let align = self.alignment_offset();
        Rect::new(
            cell.width * area.x + align.x,
            cell.height * area.y + align.y,
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
        self.calc_alignment();
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
        let width = self.size().width;
        let mut x = 0;
        let mut y = 0;
        let iter = buffer.content().iter().filter_map(|cell| {
            // XXX: numas13: do we need multi-width characters?
            let result = (!cell.skip && *cell != Cell::EMPTY).then_some((x, y, cell));
            x += 1;
            if x >= width {
                x = 0;
                y += 1;
            }
            result
        });
        self.draw(iter);
    }

    fn draw<'a>(&mut self, content: impl Iterator<Item = (u16, u16, &'a Cell)>) {
        let engine = engine();
        let ascent = self.font_map.font().ascent();
        let cell_size = self.cell_size_in_pixels();
        let align = self.alignment_offset();
        for (x, y, cell) in content {
            let x = align.x + x * cell_size.width;
            let y = align.y + y * cell_size.height;
            if cell.bg != Color::Reset {
                let p = (x as c_int, y as c_int);
                let s = (cell_size.width as c_int, cell_size.height as c_int);
                engine.fill_rgba(p, s, color_bg(cell.bg));
            }
            let y = y + ascent;
            if cell.modifier.contains(Modifier::UNDERLINED) {
                let p = (x as c_int, (y + 1) as c_int);
                let s = (cell_size.width as c_int, 2);
                engine.fill_rgba(p, s, color_fg(cell.fg));
            }
            if !cell.symbol().trim().is_empty() {
                self.cells.push((x, y, color_fg(cell.fg).into(), cell));
            }
        }

        // sorting by color results in less state changes and better performance
        self.cells.sort_unstable_by_key(|&(_, _, fg, _)| fg);

        for (x, y, fg, cell) in self.cells.drain(..) {
            let cell = unsafe { &*cell };
            for c in cell.symbol().chars() {
                let (pic, info) = self.font_map.get(c, cell.modifier);
                pic.set_with_color(fg);
                let gx = x as c_int + info.bearing_x as c_int;
                let gy = y as c_int + info.bearing_y as c_int;
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

fn convert_color(color: Color, is_fg: bool) -> [u8; 3] {
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
        Color::Rgb(r, g, b) => u32::from_le_bytes([0, r, g, b]),
        Color::Indexed(_) => todo!(),
    }
    .to_le_bytes();
    [color[2], color[1], color[0]]
}

fn color_bg(color: Color) -> [u8; 3] {
    convert_color(color, false)
}

fn color_fg(color: Color) -> [u8; 3] {
    convert_color(color, true)
}
