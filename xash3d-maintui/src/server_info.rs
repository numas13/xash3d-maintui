use core::{fmt, str::FromStr};

use compact_str::{CompactString, ToCompactString};
use xash3d_protocol::color::trim_color;
use xash3d_ui::raw::netadr_s;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Protocol {
    Xash(u8),
    GoldSrc,
}

impl Protocol {
    pub fn is_legacy(&self) -> bool {
        matches!(self, Self::Xash(48))
    }

    // pub fn is_goldsrc(&self) -> bool {
    //     matches!(self, Self::GoldSrc)
    // }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Xash(49)
    }
}

pub struct InvalidProtocolError;

impl FromStr for Protocol {
    type Err = InvalidProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "49" => Ok(Self::Xash(49)),
            "48" | "legacy" => Ok(Self::Xash(48)),
            "gs" | "goldsrc" => Ok(Self::GoldSrc),
            _ => Err(InvalidProtocolError),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xash(p) => p.fmt(f),
            Self::GoldSrc => "gs".fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct ServerInfo {
    pub addr: netadr_s,
    pub host: CompactString,
    pub host_cmp: CompactString,
    pub map: CompactString,
    pub gamedir: CompactString,
    pub numcl: u32,
    pub maxcl: u32,
    pub dm: bool,
    pub team: bool,
    pub coop: bool,
    pub password: bool,
    pub dedicated: bool,
    pub protocol: Protocol,
}

impl ServerInfo {
    // FIXME: netadr_s does not implement Default trait
    fn new(addr: netadr_s) -> Self {
        Self {
            addr,
            host: CompactString::default(),
            host_cmp: CompactString::default(),
            map: CompactString::default(),
            gamedir: CompactString::default(),
            numcl: 0,
            maxcl: 0,
            dm: false,
            team: false,
            coop: false,
            password: false,
            dedicated: false,
            protocol: Protocol::default(),
        }
    }

    pub fn with_host_and_proto(
        addr: netadr_s,
        host: impl ToCompactString,
        protocol: Protocol,
    ) -> Self {
        ServerInfo {
            host: host.to_compact_string(),
            protocol,
            ..Self::new(addr)
        }
    }

    pub fn parse(addr: netadr_s, info: &str) -> Option<Self> {
        if !info.starts_with("\\") {
            return None;
        }

        let mut ret = Self::new(addr);
        let mut it = info[1..].split('\\');
        while let Some(key) = it.next() {
            let value = it.next()?;
            match key {
                "p" => {
                    ret.protocol = trim_color(value)
                        .parse()
                        .map(Protocol::Xash)
                        .unwrap_or_default()
                }
                "host" => ret.host = value.trim().into(),
                "map" => ret.map = trim_color(value).into(),
                "gamedir" => ret.gamedir = trim_color(value).into(),
                "numcl" => ret.numcl = trim_color(value).parse().unwrap_or_default(),
                "maxcl" => ret.maxcl = trim_color(value).parse().unwrap_or_default(),
                "legacy" => {
                    if value == "1" {
                        ret.protocol = Protocol::Xash(48);
                    }
                }
                "gs" => {
                    if value == "1" {
                        ret.protocol = Protocol::GoldSrc;
                    }
                }
                "dm" => ret.dm = value == "1",
                "team" => ret.team = value == "1",
                "coop" => ret.coop = value == "1",
                "password" => ret.password = value == "1",
                "dedicated" => ret.dedicated = value == "1",
                _ => debug!("unimplemented server info {key}={value}"),
            }
        }
        ret.host_cmp = trim_color(&ret.host).to_lowercase().into();
        Some(ret)
    }
}
