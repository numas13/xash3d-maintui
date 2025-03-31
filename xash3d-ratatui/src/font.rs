use std::ffi::CString;

use freetype::{
    face::{LoadFlag, StyleFlag},
    GlyphSlot, Library,
};
use ratatui::style::Modifier;
use xash3d_ui::{engine, picture::Picture};

use crate::bmp::{Bmp, Components};

type Face = freetype::Face<&'static [u8]>;

// const FONTS: &[&str] = &[
//     "fonts/DepartureMono-Regular.woff",
// ];

const FONT: &[u8] = include_bytes!("../fonts/DepartureMono-1.422/DepartureMono-Regular.woff");

pub struct Font {
    size: isize,
    faces: Vec<Face>,
}

impl Font {
    pub fn new(size: isize) -> Self {
        let freetype = Library::init().unwrap();
        let mut faces = Vec::new();

        // for font in FONTS {
        //     let face = freetype.new_face(font, 0).unwrap();
        //     face.set_char_size(0, size * 64, 0, 0).unwrap();
        //     faces.push(face);
        // }

        let face = freetype.new_memory_face2(FONT, 0).unwrap();
        face.set_char_size(0, size * 64, 0, 0).unwrap();
        faces.push(face);

        Self { size, faces }
    }

    pub fn size(&self) -> isize {
        self.size
    }

    pub fn glyph_size(&self) -> (u16, u16) {
        let (w, h) = if let Some(metrics) = self.faces[0].size_metrics() {
            (metrics.max_advance as u32, metrics.height as u32)
        } else {
            (
                self.faces[0].max_advance_width() as u32,
                self.faces[0].height() as u32,
            )
        };
        let w = (w / 64 + 1) as u16;
        let h = (h / 64 + 1) as u16;
        (w, h)
    }

    fn find_face(&self, modifier: Modifier) -> &Face {
        let mut style = StyleFlag::empty();
        if modifier.contains(Modifier::BOLD) {
            style.insert(StyleFlag::BOLD);
        }
        if modifier.contains(Modifier::ITALIC) {
            style.insert(StyleFlag::ITALIC);
        }
        self.faces
            .iter()
            .find(|i| i.style_flags().contains(style))
            .unwrap_or(&self.faces[0])
    }

    pub fn find_glyph(&self, c: char, modifier: Modifier) -> Option<Glyph> {
        let face = self.find_face(modifier);
        if face
            .load_char(c as usize, LoadFlag::DEFAULT | LoadFlag::RENDER)
            .is_err()
        {
            error!("failed to load char '{c}' \\u{:04x}", c as u32);
            return None;
        }
        Some(Glyph { slot: face.glyph() })
    }
}

pub struct Glyph<'a> {
    slot: &'a GlyphSlot,
}

impl Glyph<'_> {
    pub fn bitmap(&self) -> Bitmap {
        Bitmap {
            bitmap: self.slot.bitmap(),
        }
    }

    pub fn horizontal_bearing(&self) -> (i16, i16) {
        let metrics = self.slot.metrics();
        let x = metrics.horiBearingX / 64;
        let y = metrics.horiBearingY / 64;
        (x as i16, y as i16)
    }
}

pub struct Bitmap {
    bitmap: freetype::Bitmap,
}

impl Bitmap {
    pub fn width(&self) -> u16 {
        self.bitmap.width() as u16
    }

    pub fn height(&self) -> u16 {
        self.buffer()
            .len()
            .checked_div(self.width() as usize)
            .unwrap_or(0) as u16
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width(), self.height())
    }

    pub fn buffer(&self) -> &[u8] {
        self.bitmap.buffer()
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
    fn new(x: u16, y: u16, glyph: &Glyph) -> Self {
        let (w, h) = glyph.bitmap().size();
        let (bearing_x, bearing_y) = glyph.horizontal_bearing();
        GlyphInfo {
            x,
            y,
            w,
            h,
            bearing_x,
            bearing_y,
        }
    }
}

pub struct GlyphMap {
    start: u32,
    pic: Picture<CString>,
    slots: Box<[GlyphInfo; Self::SIZE]>,
}

impl GlyphMap {
    const SIZE: usize = 256;

    pub fn new(font: &Font, modifier: Modifier, start: u32) -> Self {
        let (gw, gh) = font.glyph_size();
        let mut slots = Box::new([GlyphInfo::default(); Self::SIZE]);
        let width = 32 * gw;
        let height = (Self::SIZE / 32) as u16 * gh;
        let mut bmp = Bmp::builder(width, height)
            .components(Components::RGBA)
            .build();
        let mut x = 0;
        let mut y = 0;
        let end = start + Self::SIZE as u32;
        trace!("generate glyph map for {start:04x}:{end:04x}");
        for i in 0..Self::SIZE {
            let n = start + i as u32;
            let Some(glyph) = char::from_u32(n)
                .and_then(|c| {
                    if c.is_control() {
                        return None;
                    }
                    // trace!("generate glyph for '\\u{n:04x}' '{c}'");
                    font.find_glyph(c, modifier)
                })
                .or_else(|| {
                    // trace!("using fallback glyph for '\\u{n:04x}'");
                    font.find_glyph('\u{25a1}', modifier)
                })
            else {
                continue;
            };
            let bitmap = glyph.bitmap();
            let (w, h) = bitmap.size();
            if x + w > width {
                x = 0;
                y += gh;
            }
            bmp.fill_glyph(x, y, w, h, bitmap.buffer());
            slots[i] = GlyphInfo::new(x, y, &glyph);
            x += w;
        }

        // for (i, info) in slots.iter().enumerate() {
        //     let i = start + i as u32;
        //     let (x, y) = (info.x, info.y);
        //     let (w, h) = (info.w, info.h);
        //     let (bx, by) = (info.bearing_x, info.bearing_y);
        //     trace!("'\\u{i:04x}' {x:3}x{y:<3} {w:3}x{h:<3} {bx:3}x{by:<3x}");
        // }

        let mut path = format!("#mainui/backend/map{start:04x}");
        if modifier.contains(Modifier::BOLD) {
            path.push_str("_bold");
        }
        if modifier.contains(Modifier::ITALIC) {
            path.push_str("_italic");
        }
        path.push_str(".bmp");

        // {
        //     let path = format!("/tmp/map{start:04x}.bmp");
        //     std::fs::write(path, bmp.as_slice()).unwrap();
        // }

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
    maps: [Vec<GlyphMap>; 4],
}

impl FontMap {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            maps: Default::default(),
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn glyph_size(&self) -> (u16, u16) {
        self.font.glyph_size()
    }

    pub fn get(&mut self, c: char, modifier: Modifier) -> (&Picture<CString>, &GlyphInfo) {
        let mut index = 0;
        if modifier.contains(Modifier::BOLD) {
            index |= 1;
        }
        if modifier.contains(Modifier::ITALIC) {
            index |= 2;
        }
        let map = &mut self.maps[index];

        let start = c as u32 & !(GlyphMap::SIZE as u32 - 1);
        let index = match map.binary_search_by_key(&start, |i| i.start) {
            Ok(index) => index,
            Err(index) => {
                let info = GlyphMap::new(&self.font, modifier, start);
                map.insert(index, info);
                index
            }
        };
        let info = &map[index];
        (&info.pic, info.get(c as usize & (GlyphMap::SIZE - 1)))
    }
}
