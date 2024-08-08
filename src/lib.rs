#![no_std]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
mod commands;
mod device;
#[cfg(feature = "text")]
mod text;
mod utils;

pub use device::{Lcd, LcdError, LcdOrientation};
pub use utils::{rgb_to_u16, rgb_to_u8};
