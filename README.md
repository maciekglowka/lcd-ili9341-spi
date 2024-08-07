# ILI9341 LCD Spi driver

LCD driver for Waveshare 2,4" board, based on ILI9341.

`no_std` and
`embedded_hal=1.0` compatibile.

Currently supported functionalities:

- Rect / line drawing
- Sprite rendering
- Basic text rendering (via `text` feature)
- Backlight setting (via a PWM pin)

## Arduino Example

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
    // Draw magenta colored rect
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