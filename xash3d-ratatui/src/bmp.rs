use core::ffi::CStr;

use log::Level::Trace;
use xash3d_ui::picture::Picture;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Components {
    RGB = 3,
    RGBA = 4,
}

pub struct Builder {
    width: u16,
    height: u16,
    components: Components,
}

impl Builder {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            components: Components::RGB,
        }
    }

    pub fn components(mut self, components: Components) -> Self {
        self.components = components;
        self
    }

    pub fn build(self) -> Bmp {
        let offset = 0xe + 40;
        let row_size = (self.width as u32 * self.components as u32 + 3) & !3;
        let data_size = row_size * self.height as u32;
        let total_size = offset + data_size;

        let mut bmp = Bmp {
            width: self.width,
            height: self.height,
            offset: offset as usize,
            row_size,
            components: self.components,
            data: vec![0; total_size as usize],
        };

        // File Header
        bmp.write_array(0x00, b"BM");
        bmp.write_u32(0x02, total_size);
        bmp.write_u32(0x0a, offset); // image data offset

        // Info Header
        bmp.write_u32(0x0e, 40); // BITMAPINFOHEADER
        bmp.write_u32(0x12, self.width as u32);
        bmp.write_u32(0x16, self.height as u32);
        bmp.write_u16(0x1a, 1); // the number of color planes
        bmp.write_u16(0x1c, self.components as u16 * 8); // the number of bits per pixel
        bmp.write_u32(0x1e, 0); // compression
        bmp.write_u32(0x22, 0); // size of the raw bitmap data
        bmp.write_u32(0x26, 0); // the horizontal resolution of the image
        bmp.write_u32(0x2a, 0); // the vertical resolution of the image
        bmp.write_u32(0x2e, 0); // the number of colors in the color palette
        bmp.write_u32(0x32, 0); // the number of important colors used

        bmp
    }
}

pub struct Bmp {
    width: u16,
    height: u16,
    row_size: u32,
    components: Components,
    offset: usize,
    data: Vec<u8>,
}

impl Bmp {
    pub fn builder(width: u16, height: u16) -> Builder {
        Builder::new(width, height)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline(always)]
    fn write_array<const N: usize>(&mut self, offset: usize, bytes: &[u8; N]) {
        for (i, &byte) in bytes.iter().enumerate() {
            self.data[offset + i] = byte;
        }
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.write_array(offset, &value.to_le_bytes());
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.write_array(offset, &value.to_le_bytes());
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, r: u8, g: u8, b: u8, a: u8) {
        if log_enabled!(Trace) && (x >= self.width() || y >= self.height()) {
            let (w, h) = (self.width(), self.height());
            trace!("Bmp::set_pixel invalid coord {x}x{y} for size {w}x{h}");
            return;
        }
        let x = x as usize * self.components as usize;
        let offset = self.offset + self.row_size as usize * (y as usize) + x;
        match self.components {
            Components::RGB => self.write_array(offset, &[r, g, b]),
            Components::RGBA => self.write_array(offset, &[r, g, b, a]),
        }
    }

    pub fn fill_glyph(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) {
        let y = self.height - y - h;
        let mut bytes = data.iter();
        for y in (y..y + h).rev() {
            for x in x..x + w {
                match bytes.next() {
                    Some(&c) => self.set_pixel(x, y, 255, 255, 255, c),
                    None => {
                        warn!("Bmp::fill_rect: data is less then rect");
                        return;
                    }
                }
            }
        }
    }

    pub fn create_picture<S: AsRef<CStr>>(&self, path: S) -> Picture<S> {
        // TODO: return Result
        assert!(path.as_ref().to_bytes().ends_with(b".bmp"));
        Picture::create(path, self.as_slice())
    }
}
