use core::{fmt::Write, str};

use alloc::{string::String, vec::Vec};
use xash3d_ui::{engine, parser::Tokens, raw::netadr_s};

use crate::server_info::Protocol;

pub struct SavedServer {
    addr: netadr_s,
    protocol: Protocol,
}

impl SavedServer {
    pub fn new(addr: netadr_s, protocol: Protocol) -> Self {
        Self { addr, protocol }
    }

    pub fn addr(&self) -> &netadr_s {
        &self.addr
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
}

#[derive(Default)]
pub struct SavedServers {
    list: Vec<SavedServer>,
    changed: bool,
}

impl SavedServers {
    pub fn load_from_file(path: &str) -> Result<Self, &'static str> {
        let engine = engine();
        let file = engine.load_file(path).ok_or("failed to load")?;
        let data = str::from_utf8(file.as_slice()).map_err(|_| "invalid utf8")?;
        let mut tokens = Tokens::new(data).handle_colon(false);
        let mut servers = Self::default();
        while let Some((Ok(addr_raw), Ok(protocol))) = tokens.next().zip(tokens.next()) {
            let Some(addr) = engine.string_to_addr(addr_raw) else {
                warn!("invalid address {addr_raw:?} in file \"{path}\"");
                continue;
            };
            let protocol = match protocol.parse() {
                Ok(protocol) => protocol,
                Err(_) => {
                    warn!("invalid protocol {protocol} for {addr_raw:?} in file \"{path}\"");
                    continue;
                }
            };
            if !servers.contains(&addr) {
                servers.list.push(SavedServer::new(addr, protocol));
            }
        }
        trace!("load {} servers from file \"{path}\"", servers.list.len());
        Ok(servers)
    }

    pub fn save_to_file(&self, path: &str) {
        if !self.changed {
            return;
        }
        let engine = engine();
        let mut out = String::new();
        let mut count = 0;
        for i in &self.list {
            count += 1;
            let address = engine.addr_to_string(i.addr);
            writeln!(&mut out, "{address} {}", i.protocol).unwrap();
        }
        if count > 0 {
            trace!("save {count} servers to file \"{path}\"");
            engine.save_file(path, out.as_bytes());
        } else {
            trace!("delete servers file \"{path}\"");
            engine.remove_file(path);
        }
    }

    pub fn insert(&mut self, addr: netadr_s, protocol: Protocol) -> Option<&SavedServer> {
        if !self.contains(&addr) {
            self.changed = true;
            self.list.push(SavedServer::new(addr, protocol));
            self.list.last()
        } else {
            None
        }
    }

    pub fn remove(&mut self, addr: &netadr_s) -> Option<SavedServer> {
        let engine = engine();
        self.list
            .iter()
            .position(|i| engine.compare_addr(&i.addr, addr))
            .map(|i| {
                self.changed = true;
                self.list.remove(i)
            })
    }

    pub fn contains(&self, addr: &netadr_s) -> bool {
        let engine = engine();
        self.list.iter().any(|i| engine.compare_addr(&i.addr, addr))
    }

    pub fn iter(&self) -> impl Iterator<Item = &SavedServer> {
        self.list.iter()
    }
}
