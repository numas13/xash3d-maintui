#![no_std]

#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
compile_error!("std feature is required");

#[macro_use]
extern crate log;

mod backend;
mod bmp;
mod font;

pub use backend::XashBackend;
