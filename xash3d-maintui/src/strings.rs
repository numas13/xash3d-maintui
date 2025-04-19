use std::{borrow::Cow, char, collections::HashMap, ffi::CStr, fmt::Write, str};

use csz::CStrArray;
use xash3d_cell::SyncOnceCell;
use xash3d_shared::{
    cvar::CVarFlags,
    parser::{Error, Tokens},
};
use xash3d_ui::engine;

const DEFAULT_LANGUAGE: &str = "english";
const CUSTOM_STRINGS_PATH: &CStr = c"gfx/shell/strings.lst";

const UI_LANGUAGE: &CStr = c"ui_language";

#[derive(Default)]
pub struct Strings {
    map: HashMap<String, String>,
}

impl Strings {
    fn new() -> Self {
        let mut strings = Self::default();
        strings.load();
        strings
    }

    fn load(&mut self) {
        if let Some(_file) = engine().load_file(CUSTOM_STRINGS_PATH) {
            warn!("unimplemented {CUSTOM_STRINGS_PATH:?}");
        }

        let eng = engine();
        eng.client_cmd_now(c"exec mainui.cfg\n");

        self.load_language(DEFAULT_LANGUAGE);
        let lang = eng.get_cvar_string(UI_LANGUAGE);
        if let Ok(lang) = lang.to_str() {
            if lang != DEFAULT_LANGUAGE {
                self.load_language(lang);
            }
        }
    }

    fn load_language(&mut self, lang: &str) {
        trace!("load strings for language \"{lang}\"");
        let info = engine().get_game_info_2().unwrap();
        let gamedir = info.gamefolder.as_c_str().to_str().unwrap();
        for i in ["gameui", "valve", "mainui", "maintui"] {
            if i != gamedir {
                self.load_gamedir(i, lang);
            }
        }

        self.load_gamedir(gamedir, lang);
    }

    fn load_gamedir(&mut self, gamedir: &str, lang: &str) {
        let engine = engine();
        let mut path = CStrArray::<128>::new();
        path.cursor()
            .write_fmt(format_args!("resource/{gamedir}_{lang}.txt"))
            .unwrap();

        let Some(file) = engine.load_file(&path) else {
            error!("failed to open {path}");
            return;
        };

        let src = bytes_to_string(file.as_slice());
        if let Err(err) = self.parse_resource_file(&src) {
            error!("failed to parse {path}, {err:?}");
        }
    }

    fn parse_resource_file<'a>(&mut self, src: &'a str) -> Result<(), Error<'a>> {
        let mut tokens = Tokens::new(src);
        tokens.expect("lang")?;
        tokens.expect("{")?;
        tokens.expect("Language")?;
        let _language = tokens.parse()?;
        tokens.expect("Tokens")?;
        tokens.expect("{")?;
        loop {
            let name = tokens.parse()?;
            if name == "}" {
                break;
            }
            let value = escape_string(tokens.parse()?);
            if !value.is_empty() {
                self.map.insert(name.to_string(), value);
            }
        }
        tokens.expect("}")?;
        Ok(())
    }

    pub fn try_get<'a>(&'a self, s: &'a str) -> Option<&'a str> {
        self.map
            .get(s.strip_prefix("#").unwrap_or(s))
            .map(|v| v.as_str())
    }

    pub fn get<'a>(&'a self, s: &'a str) -> &'a str {
        self.try_get(s).unwrap_or(s)
    }
}

static STRINGS: SyncOnceCell<Strings> = unsafe { SyncOnceCell::new() };

pub fn init() {
    engine().register_variable(UI_LANGUAGE, DEFAULT_LANGUAGE, CVarFlags::ARCHIVE);
}

pub fn strings() -> &'static Strings {
    STRINGS.get_or_init(Strings::new)
}

// pub fn try_get(s: &str) -> Option<&str> {
//     strings().try_get(s)
// }

pub fn get(s: &str) -> &str {
    strings().get(s)
}

fn from_utf32_lossy(data: &[u8], be: bool) -> String {
    let mut buf = String::with_capacity(data.len() / 4);
    for chunk in data.chunks_exact(4) {
        let mut arr = [0; 4];
        arr.copy_from_slice(chunk);
        let i = if be {
            u32::from_be_bytes(arr)
        } else {
            u32::from_le_bytes(arr)
        };
        let c = char::from_u32(i).unwrap_or(char::REPLACEMENT_CHARACTER);
        buf.push(c);
    }
    buf
}

fn from_utf16_lossy(data: &[u8], be: bool) -> String {
    let mut buf = String::with_capacity(data.len() / 2);
    let iter = data.chunks_exact(2).map(|chunk| {
        let mut arr = [0; 2];
        arr.copy_from_slice(chunk);
        if be {
            u16::from_be_bytes(arr)
        } else {
            u16::from_le_bytes(arr)
        }
    });
    for c in char::decode_utf16(iter) {
        buf.push(c.unwrap_or(char::REPLACEMENT_CHARACTER));
    }
    buf
}

fn bytes_to_string(data: &[u8]) -> Cow<'_, str> {
    match data {
        [0x00, 0x00, 0xfe, 0xff, ..] => from_utf32_lossy(&data[4..], true).into(),
        [0xfe, 0xff, 0x00, 0x00, ..] => from_utf32_lossy(&data[4..], false).into(),
        [0xfe, 0xff, ..] => from_utf16_lossy(&data[2..], true).into(),
        [0xff, 0xfe, ..] => from_utf16_lossy(&data[2..], false).into(),
        [0xef, 0xbb, 0xbf, ..] => String::from_utf8_lossy(&data[3..]),
        _ => String::from_utf8_lossy(data),
    }
}

fn escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut escape = false;
    for mut c in s.chars() {
        if escape {
            escape = false;
            match c {
                '\\' => c = '\\',
                'n' => c = '\n',
                _ => out.push('\\'),
            }
        } else if c == '\\' {
            escape = true;
            continue;
        }
        out.push(c);
    }
    out
}

pub trait Localize {
    fn localize(&self) -> &str;
}

impl Localize for str {
    fn localize(&self) -> &str {
        strings().get(self)
    }
}
