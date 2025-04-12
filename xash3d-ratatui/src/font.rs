use std::ffi::CString;

use ab_glyph::{Font as _, FontRef, PxScaleFont, ScaleFont};
use ratatui::style::Modifier;
use xash3d_ui::{engine, picture::Picture};

use crate::bmp::{Bmp, Components};

const FONT: &[u8] = include_bytes!("../fonts/DepartureMono-1.422/DepartureMono-Regular.otf");

pub struct Font {
    font: PxScaleFont<FontRef<'static>>,
}

impl Font {
    pub fn new(size: isize) -> Self {
        Self {
            font: FontRef::try_from_slice(FONT)
                .unwrap()
                .into_scaled(size as f32),
        }
    }

    pub fn size(&self) -> isize {
        self.font.scale.y as isize
    }

    pub fn ascent(&self) -> u16 {
        self.font.ascent() as u16
    }

    pub fn glyph_size(&self) -> (u16, u16) {
        let w = self.font.h_advance(self.font.glyph_id('_')) as u16;
        let h = self.font.height() as u16;
        (w, h)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct GlyphInfo {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub bearing_x: i16,
    pub bearing_y: i16,
}

impl GlyphInfo {
    fn new(x: u16, y: u16, bounds: &ab_glyph::Rect) -> Self {
        GlyphInfo {
            x,
            y,
            w: bounds.width() as u16,
            h: bounds.height() as u16,
            bearing_x: bounds.min.x as i16,
            bearing_y: bounds.min.y as i16,
        }
    }
}

struct GlyphMap {
    start: u32,
    pic: Picture<CString>,
    slots: Box<[GlyphInfo; Self::SIZE]>,
}

impl GlyphMap {
    const SIZE: usize = 256;

    fn new(font: &Font, start: u32) -> Self {
        let (gw, gh) = font.glyph_size();
        let font = &font.font;
        let mut slots = Box::new([GlyphInfo::default(); Self::SIZE]);
        let width = 32 * gw;
        let height = (Self::SIZE / 32) as u16 * gh;
        let mut bmp = Bmp::builder(width, height)
            .components(Components::RGBA)
            .build();
        let (mut x, mut y) = (0, 0);
        let end = start + Self::SIZE as u32;
        trace!("generate glyph map for {start:04x}:{end:04x}");
        for i in 0..Self::SIZE {
            let n = start + i as u32;
            let c = match char::from_u32(n) {
                Some(c) if c.is_control() => continue,
                Some(c) => c,
                None => continue,
            };
            // trace!("generate glyph for '\\u{n:04x}' '{c}'");
            let glyph = font.scaled_glyph(c);
            let outline = font
                .outline_glyph(glyph)
                .or_else(|| font.outline_glyph(font.scaled_glyph('\u{25a1}')));
            let Some(outline) = outline else {
                trace!("skip glyph for '\\u{n:04x}' '{c}'");
                continue;
            };
            let bounds = outline.px_bounds();
            let w = bounds.width() as u16;
            if x + w > width {
                x = 0;
                y += gh;
            }
            outline.draw(|px, py, f| {
                if f <= 0.0 {
                    return;
                }
                let a = (f * 255.0) as u8;
                let x = x + px as u16;
                let y = bmp.height() - (y + py as u16) - 1;
                bmp.set_pixel(x, y, 255, 255, 255, a);
            });
            slots[i] = GlyphInfo::new(x, y, &bounds);
            x += w;
        }

        if false {
            for (i, info) in slots.iter().enumerate() {
                let i = start + i as u32;
                let (x, y) = (info.x, info.y);
                let (w, h) = (info.w, info.h);
                let (bx, by) = (info.bearing_x, info.bearing_y);
                trace!("'\\u{i:04x}' {x:3}x{y:<3} {w:3}x{h:<3} {bx:3}x{by:<3x}");
            }
        }

        let path = format!("#mainui/backend/map{start:04x}.bmp");

        if false {
            let path = format!("/tmp/map{start:04x}.bmp");
            std::fs::write(path, bmp.as_slice()).unwrap();
        }

        let pic = bmp.create_picture(CString::new(path).unwrap());
        Self { start, pic, slots }
    }

    fn get(&self, i: usize) -> &GlyphInfo {
        &self.slots[i]
    }
}

impl Drop for GlyphMap {
    fn drop(&mut self) {
        // FIXME: RAII for pictures are disabled globally because the engine uses one global
        // id for all pictures with same path. Required manual free.
        engine().pic_free(self.pic.path());
    }
}

pub struct FontMap {
    font: Font,
    map: Vec<GlyphMap>,
}

impl FontMap {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            map: Default::default(),
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn glyph_size(&self) -> (u16, u16) {
        self.font.glyph_size()
    }

    pub fn get(&mut self, c: char, _: Modifier) -> (&Picture<CString>, &GlyphInfo) {
        let start = c as u32 & !(GlyphMap::SIZE as u32 - 1);
        let index = match self.map.binary_search_by_key(&start, |i| i.start) {
            Ok(index) => index,
            Err(index) => {
                let info = GlyphMap::new(&self.font, start);
                self.map.insert(index, info);
                index
            }
        };
        let info = &self.map[index];
        (&info.pic, info.get(c as usize & (GlyphMap::SIZE - 1)))
    }
}
