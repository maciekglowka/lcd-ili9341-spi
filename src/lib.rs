#![no_std]
use embedded_hal as hal;

use hal::digital::OutputPin;
use hal::spi::SpiDevice;

mod commands;
mod device;
mod text;
mod utils;

pub use device::{Lcd, LcdOrientation};
pub use utils::rgb_to_u16;

