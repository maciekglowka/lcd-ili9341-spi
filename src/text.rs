use embedded_hal as hal;

use hal::digital::OutputPin;
use hal::pwm::SetDutyCycle;
use hal::spi::SpiBus;

use crate::device::{Lcd, LcdError};
use crate::utils::u16_to_bytes;

const FONT: &[u8] = include_bytes!("../assets/font.bin");
const CHAR_START: usize = 0x20;

impl<T, U, V, W> Lcd<T, U, V, W> 
where
    T: SpiBus,
    U: OutputPin,
    V: OutputPin,
    W: SetDutyCycle
{
    /// Expects single-byte per character.
    /// Supports only base ASCII up to code 128.
    pub fn draw_text(&mut self, x: u16, y: u16, text: &str, fg_color: u16, bg_color: u16, scale: u16) -> Result<(), LcdError> {
        let mut cx = x;

        for c in text.bytes() {
            let offset = 8 * (c as usize - CHAR_START);
            let data: [u8; 8] = FONT[offset..offset + 8].try_into().unwrap();
            self.draw_character(cx, y, &data, fg_color, bg_color, scale)?;
            cx += scale * 8
        }
        Ok(())
    }

    fn draw_character(&mut self, x: u16, y: u16, data: &[u8; 8], fg_color: u16, bg_color: u16, scale: u16) -> Result<(), LcdError> {
        self.set_window(x, y, x + scale * 8, y + scale * 8)?;

        let (fgh, fgl) = u16_to_bytes(fg_color);
        let (bgh, bgl) = u16_to_bytes(bg_color);

        let mut buffer = [0; 16];
        let chunk = 8 / scale;

        self.enable_write_data()?;
        
        for row in 0..8 {
            for _ in 0..scale {
                for bit_offset in (0..scale).rev() {
                    let bit_start = bit_offset * chunk;
                    let bit_end = (bit_offset + 1) * chunk;
                    for i in bit_start..bit_end {
                        let (h, l) = if data[row] >> i & 1 == 0 {
                            (bgh, bgl)
                        } else {
                            (fgh, fgl)
                        };
                        let mut offset = 16 - ((i - bit_start + 1) * 2 * scale) as usize;
                        for b in 0..scale as usize {
                            buffer[offset + 2 * b + 1] = l;
                            buffer[offset + 2 * b] = h;
                        }
                    }
                    self.write_data_continue(&buffer)?;
                }
            }
        }
        Ok(())
    }
}
