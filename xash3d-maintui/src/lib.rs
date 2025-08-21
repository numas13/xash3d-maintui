#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
extern crate log;

macro_rules! define_strings {
    ($($name:ident { $($body:tt)* })*) => {
        $(pub mod $name {
            define_strings!($($body)*);
        })*
    };
    ($($name:ident = $value:expr),* $(,)?) => {
        $(pub const $name: &str = $value;)*
    };
}

mod config_list;
mod export;
mod i18n;
mod input;
mod macros;
mod menu;
mod saved_servers;
mod server_info;
mod strings;
mod ui;
mod widgets;
