use embedded_hal as hal;

use hal::delay::DelayNs;
use hal::digital::OutputPin;
use hal::pwm::SetDutyCycle;
use hal::spi::SpiBus;

use crate::commands::*;
use crate::utils::u16_to_bytes;

const COLUMNS: u16 = 240;
const PAGES: u16 = 320;

pub enum LcdError {
    PinError,
    SpiError,
}

/// Display rotation, where Rotate0 is the default vertical orientation.
/// Enum variants represent clock-wise rotation angles.
pub enum LcdOrientation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

/// Main LCD struct
///
/// Required hardware connections:
/// - spi: the Spi interface
/// - dc_pin: Data / Command selector
/// - rst_pin: Device reset pin
/// - bl_pin: backlight level pin (PWM)
pub struct Lcd<T, U, V, W> {
    spi: T,
    dc_pin: U,  // Data / Command - 0=WriteCommand, 1=WriteData
    rst_pin: V, // Reset
    bl_pin: W,  // Backlight PWM
    orientation: LcdOrientation,
}

impl<T, U, V, W> Lcd<T, U, V, W>
where
    T: SpiBus,
    U: OutputPin,
    V: OutputPin,
    W: SetDutyCycle,
{
    pub fn new(spi: T, dc_pin: U, rst_pin: V, bl_pin: W) -> Self {
        Self {
            spi,
            dc_pin,
            rst_pin,
            bl_pin,
            orientation: LcdOrientation::Rotate0,
        }
    }
    /// Sets display's rotation
    ///
    /// Should be called before LCD initialization:
    /// ```
    /// let mut lcd = Lcd::new(
    ///     spi,
    ///     dc_pin,
    ///     rst_pin,
    ///     bl_pin
    ///    );
    ///    .with_orientation(lcd_2inch4::LcdOrientation::Rotate90);
    /// let _ = lcd.init(&mut delay);
    /// ```

    pub fn with_orientation(mut self, orientation: LcdOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    fn size(&self) -> (u16, u16) {
        match self.orientation {
            LcdOrientation::Rotate0 | LcdOrientation::Rotate180 => (COLUMNS, PAGES),
            LcdOrientation::Rotate90 | LcdOrientation::Rotate270 => (PAGES, COLUMNS),
        }
    }

    fn memory_access_control_value(&self) -> u8 {
        let orientation = match self.orientation {
            LcdOrientation::Rotate0 => 0b00000000,
            LcdOrientation::Rotate90 => 0b01100000,
            LcdOrientation::Rotate180 => 0b11000000,
            LcdOrientation::Rotate270 => 0b10100000,
        };
        orientation | 0b00001000
    }

    pub fn reset<D>(&mut self, delay: &mut D) -> Result<(), LcdError>
    where
        D: DelayNs,
    {
        // delay values taken from the Arduino C driver
        delay.delay_ms(200);
        self.rst_pin.set_low().map_err(|_| LcdError::PinError)?;
        delay.delay_ms(200);
        self.rst_pin.set_high().map_err(|_| LcdError::PinError)?;
        delay.delay_ms(200);
        Ok(())
    }

    pub fn set_backlight(&mut self, value: u16) -> Result<(), LcdError> {
        self.bl_pin
            .set_duty_cycle(value)
            .map_err(|_| LcdError::PinError)?;
        Ok(())
    }

    fn write_command(&mut self, cmd: u8) -> Result<(), LcdError> {
        self.dc_pin.set_low().map_err(|_| LcdError::PinError)?;
        self.spi.write(&[cmd]).map_err(|_| LcdError::SpiError)?;
        Ok(())
    }

    /// Sets the data pin
    #[inline(always)]
    pub(crate) fn enable_write_data(&mut self) -> Result<(), LcdError> {
        self.dc_pin.set_high().map_err(|_| LcdError::PinError)?;
        Ok(())
    }

    /// Sets the data pin and sends the payload
    #[inline(always)]
    pub(crate) fn write_data(&mut self, data: &[u8]) -> Result<(), LcdError> {
        self.enable_write_data()?;
        self.write_data_continue(data)?;
        Ok(())
    }

    /// Sends the payload via SPI.
    /// Expects to be called only after `enable_write_data` or `write_data`
    #[inline(always)]
    pub(crate) fn write_data_continue(&mut self, data: &[u8]) -> Result<(), LcdError> {
        self.spi.write(data).map_err(|_| LcdError::SpiError)?;
        Ok(())
    }

    pub fn init<D>(&mut self, delay: &mut D) -> Result<(), LcdError>
    where
        D: DelayNs,
    {
        // command sequence taken from Arduino C driver
        self.reset(delay)?;

        self.write_command(SLEEP_OUT)?;

        self.write_command(POWER_CONTROL_B)?;
        self.write_data(&[0x00])?;
        self.write_data(&[0xC1])?;
        self.write_data(&[0x30])?;
        self.write_command(POWER_ON_SEQ_CONTROL)?;
        self.write_data(&[0x64])?;
        self.write_data(&[0x03])?;
        self.write_data(&[0x12])?;
        self.write_data(&[0x81])?;
        self.write_command(DRIVER_TIMING_CONTROL_A)?;
        self.write_data(&[0x85])?;
        self.write_data(&[0x00])?;
        self.write_data(&[0x79])?;
        self.write_command(POWER_CONTROL_A)?;
        self.write_data(&[0x39])?;
        self.write_data(&[0x2C])?;
        self.write_data(&[0x00])?;
        self.write_data(&[0x34])?;
        self.write_data(&[0x02])?;
        self.write_command(PUMP_RATIO_CONTROL)?;
        self.write_data(&[0x20])?;
        self.write_command(DRIVER_TIMING_CONTROL_B)?;
        self.write_data(&[0x00])?;
        self.write_data(&[0x00])?;
        self.write_command(POWER_CONTROL_1)?;
        self.write_data(&[0x1D])?;
        self.write_command(POWER_CONTROL_2)?;
        self.write_data(&[0x12])?;
        self.write_command(VCOM_CONTROL_1)?;
        self.write_data(&[0x33])?;
        self.write_data(&[0x3F])?;
        self.write_command(VCOM_CONTROL_2)?;
        self.write_data(&[0x92])?;
        self.write_command(PIXEL_FORMAT_SET)?;
        self.write_data(&[0x55])?;
        self.write_command(MEMORY_ACCESS_CONTROL)?;
        self.write_data(&[self.memory_access_control_value()])?;
        self.write_command(FRAME_CONTROL_NORMAL_MODE)?;
        self.write_data(&[0x00])?;
        self.write_data(&[0x12])?;
        self.write_command(DISPLAY_FUNCTION_CONTROL)?;
        self.write_data(&[0x0A])?;
        self.write_data(&[0xA2])?;

        self.write_command(SET_TEAR_SCANLINE)?;
        self.write_data(&[0x02])?;

        self.write_command(DISPLAY_ON)?;
        self.set_gamma()?;

        self.set_backlight(255)?;

        Ok(())
    }

    fn set_gamma(&mut self) -> Result<(), LcdError> {
        self.write_command(ENABLE_3G)?;
        self.write_data(&[0x00])?;
        self.write_command(GAMMA_SET)?;
        self.write_data(&[0x01])?;
        self.write_command(POSITIVE_GAMMA_CORRECTION)?;
        self.write_data(&[0x0F])?;
        self.write_data(&[0x22])?;
        self.write_data(&[0x1C])?;
        self.write_data(&[0x1B])?;
        self.write_data(&[0x08])?;
        self.write_data(&[0x0F])?;
        self.write_data(&[0x48])?;
        self.write_data(&[0xB8])?;
        self.write_data(&[0x34])?;
        self.write_data(&[0x05])?;
        self.write_data(&[0x0C])?;
        self.write_data(&[0x09])?;
        self.write_data(&[0x0F])?;
        self.write_data(&[0x07])?;
        self.write_data(&[0x00])?;
        self.write_command(NEGATIVE_GAMMA_CORRECTION)?;
        self.write_data(&[0x00])?;
        self.write_data(&[0x23])?;
        self.write_data(&[0x24])?;
        self.write_data(&[0x07])?;
        self.write_data(&[0x10])?;
        self.write_data(&[0x07])?;
        self.write_data(&[0x38])?;
        self.write_data(&[0x47])?;
        self.write_data(&[0x4B])?;
        self.write_data(&[0x0A])?;
        self.write_data(&[0x13])?;
        self.write_data(&[0x06])?;
        self.write_data(&[0x30])?;
        self.write_data(&[0x38])?;
        self.write_data(&[0x0F])?;
        Ok(())
    }

    /// Leave off state
    pub fn display_on(&mut self) -> Result<(), LcdError> {
        self.write_command(DISPLAY_ON)
    }

    /// Enter off state
    pub fn display_off(&mut self) -> Result<(), LcdError> {
        self.write_command(DISPLAY_OFF)
    }

    /// Enter sleep mode
    pub fn enter_sleep_mode(&mut self) -> Result<(), LcdError> {
        self.write_command(ENTER_SLEEP_MODE)
    }

    /// Disable sleep mode
    pub fn leave_sleep_mode(&mut self) -> Result<(), LcdError> {
        self.write_command(SLEEP_OUT)
    }

    pub(crate) fn set_window(
        &mut self,
        x0: u16,
        y0: u16,
        x1: u16,
        y1: u16,
    ) -> Result<(), LcdError> {
        let c1 = x1.saturating_sub(1).max(x0);
        let p1 = y1.saturating_sub(1).max(y0);
        let (c0h, c0l) = u16_to_bytes(x0);
        let (c1h, c1l) = u16_to_bytes(c1);
        let (p0h, p0l) = u16_to_bytes(y0);
        let (p1h, p1l) = u16_to_bytes(p1);

        self.write_command(COLUMN_ADDRESS_SET)?;
        self.write_data(&[c0h, c0l, c1h, c1l])?;

        self.write_command(PAGE_ADDRESS_SET)?;
        self.write_data(&[p0h, p0l, p1h, p1l])?;

        self.write_command(MEMORY_WRITE)?;
        Ok(())
    }

    /// Clear the entire screen with the given color
    pub fn clear(&mut self, color: u16) -> Result<(), LcdError> {
        let (w, h) = self.size();
        self.fill_rect(0, 0, w, h, color)?;

        Ok(())
    }

    /// Draw filled rect or line (when width or height set to 1)
    pub fn fill_rect(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        color: u16,
    ) -> Result<(), LcdError> {
        self.set_window(x, y, x + w, y + h)?;
        let (ch, cl) = u16_to_bytes(color);
        self.enable_write_data()?;

        // spi send optimization
        // slight buffer overflow seems ok
        let chunk = [ch, cl, ch, cl, ch, cl, ch, cl];
        for _ in 0..(w as u32 * h as u32).div_ceil(4) {
            self.write_data_continue(&chunk)?;
        }
        Ok(())
    }

    /// Draw raw sprite data on the screen.
    ///
    /// The input buffer should contain color information in high_byte_u8, low_byte_u8 format.
    /// Buffer length should match the rect specified by the (x, y, w, h) although it's currently
    /// not checked.
    pub fn draw_sprite(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        data: &[u8],
    ) -> Result<(), LcdError> {
        self.set_window(x, y, x + w, y + h)?;
        self.write_data(&data)?;
        Ok(())
    }
}
