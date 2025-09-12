use compact_str::{CompactString, ToCompactString};
use xash3d_protocol::color::trim_color;
use xash3d_ui::engine::{net::netadr_s, Protocol};

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
                    ret.protocol = match trim_color(value).as_ref() {
                        "48" => Protocol::Xash48,
                        "49" => Protocol::Xash49,
                        _ => Protocol::Current,
                    }
                }
                "host" => ret.host = value.trim().into(),
                "map" => ret.map = trim_color(value).into(),
                "gamedir" => ret.gamedir = trim_color(value).into(),
                "numcl" => ret.numcl = trim_color(value).parse().unwrap_or_default(),
                "maxcl" => ret.maxcl = trim_color(value).parse().unwrap_or_default(),
                "legacy" => {
                    if value == "1" {
                        ret.protocol = Protocol::Xash48;
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
