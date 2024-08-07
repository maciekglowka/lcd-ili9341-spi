#![no_std]
//! LCD driver for Waveshare 2,4" board, based on ILI9341.
//!
//! `no_std` and `embedded_hal=1.0` compatibile.
//!
mod commands;
mod device;
mod text;
mod utils;

pub use device::{Lcd, LcdOrientation};
pub use utils::rgb_to_u16;
