# ILI9341 LCD Spi driver

[![crates.io](https://img.shields.io/crates/v/lcd-ili9341-spi)](https://crates.io/crates/lcd-ili9341-spi)
[![Documentation](https://img.shields.io/docsrs/lcd-ili9341-spi)](https://docs.rs/lcd-ili9341-spi) 

Device agnostic LCD driver for Waveshare 2,4" board, based on ILI9341.

`no_std` and
`embedded_hal=1.0` compatibile.

### Currently supports:

- Rect / line drawing
- Sprite buffer rendering
- Basic text rendering (via a `text` feature)
- Backlight level setting (via a PWM pin)


#### Text rendering 

As the text rendering requires an embedded font, it has been hidden behind a feature flag
in order to save memory when not needed. The font supports basic ASCII range: codes 32 - 128 (each character takes 8 bytes).


### Required hardware connections:

- SPI: impl `SpiBus` (from the embedded_hal)
- DC pin: Data / Command selector - Digital Pin
- RST pin: Device reset - Digital Pin
- BL pin: backlight level - PWM pin



## Arduino Uno Example

```rust
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;
use arduino_hal::spi;
use core::panic::PanicInfo;
use lcd_ili9341_spi::{
    Lcd, rgb_to_u16, rgb_to_u8, LcdOrientation
};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let spi_settings = spi::Settings {
        data_order: spi::DataOrder::MostSignificantFirst,
        clock: spi::SerialClockRate::OscfOver2,
        mode: embedded_hal::spi::MODE_0,
    };

    let (mut arduino_spi, _) = arduino_hal::Spi::new(
        dp.SPI,
        pins.d13.into_output(),        
        pins.d11.into_output(),
        pins.d12.into_pull_up_input(),
        pins.d10.into_output(),
        spi_settings
    );
    
    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Direct);
    let mut bl_pin = pins.d9.into_output().into_pwm(&timer1);
    bl_pin.enable();

    let mut lcd = Lcd::new(
            arduino_spi,
            pins.d7.into_output(),
            pins.d8.into_output(),
            bl_pin
        )
        .with_orientation(LcdOrientation::Rotate90);

    let mut delay = arduino_hal::Delay::new();
    let _ = lcd.init(&mut delay);

    let _ = lcd.set_backlight(140);

    // Make the screen black
    let _ = lcd.clear(0x0000);
    // Draw magenta rect
    lcd.fill_rect(10, 10, 20, 30, rgb_to_u16(255, 0, 255));
    // Draw blue-ish horizontal line
    lcd.fill_rect(0, 50, 320, 1, rgb_to_u16(0, 128, 255));

    // Draw 4x4 px red-black checker
    let (red_h, red_l) = rgb_to_u8(255, 0, 0);
    let sprite = [
        red_h, red_l, red_h, red_l, 0, 0, 0, 0,
        red_h, red_l, red_h, red_l, 0, 0, 0, 0,
        0, 0, 0, 0, red_h, red_l, red_h, red_l,
        0, 0, 0, 0, red_h, red_l, red_h, red_l,
    ];

    lcd.draw_sprite(50, 70, 4, 4, &sprite);

    loop {
        arduino_hal::delay_ms(100);
    }
}

```

## Raspberry Pi Example

```rust
use lcd_ili9341_spi::{rgb_to_u16, rgb_to_u8, Lcd, LcdOrientation};
use rppal::gpio::Gpio;
use rppal::hal::Delay;
use rppal::pwm::{Channel, Polarity, Pwm};
use rppal::spi::{Bus, Mode, Segment, SlaveSelect, Spi};

fn main() {
    let dc_pin = Gpio::new().unwrap().get(25).unwrap().into_output();
    let rst_pin = Gpio::new().unwrap().get(27).unwrap().into_output();
    let bl_pin = PwmHal(Pwm::with_frequency(Channel::Pwm0, 512.0, 0.5, Polarity::Normal, true).unwrap()); 

    let mut pi_spi = Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0,
        8_000_000,
        rppal::spi::Mode::Mode0,
    )
    .unwrap();

    let mut lcd = Lcd::new(pi_spi, dc_pin, rst_pin, bl_pin);
    let mut delay = Delay::new();
    let _ = lcd.init(&mut delay);

    let _ = lcd.set_backlight(140);

    // Make the screen black
    let _ = lcd.clear(0x0000);
    // Draw magenta colored rect
    lcd.fill_rect(10, 10, 20, 30, rgb_to_u16(255, 0, 255));
    // Render text
    lcd.draw_text(10, 120, "RustberryPi", rgb_to_u16(0, 128, 255), 0x0000, 2);

    std::thread::sleep(std::time::Duration::from_secs(15));
}

// SetDutyCycle impl is missing in the rppal crate?
struct PwmHal(Pwm);
impl embedded_hal::pwm::SetDutyCycle for PwmHal {
    fn max_duty_cycle(&self) -> u16 {
        255
    }
    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.0.set_duty_cycle(duty as f64 / 255.0);
        Ok(())
    }
}
impl embedded_hal::pwm::ErrorType for PwmHal {
    type Error = PwmError;
}

#[derive(Debug)]
struct PwmError;
impl embedded_hal::pwm::Error for PwmError {
    fn kind(&self) -> embedded_hal::pwm::ErrorKind {
        embedded_hal::pwm::ErrorKind::Other
    }
}
```