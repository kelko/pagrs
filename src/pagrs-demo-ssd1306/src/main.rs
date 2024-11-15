#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::pixelcolor::Rgb565;
use heapless::String;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};
use ufmt::uwrite;

#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

use pagrs::Screensaver;
use pagrs::{HorizontalAlignment, StaticImage, VerticalAlignment};
use pagrs::PageRotator;
use pagrs::DigitalRain;
use pagrs::{DynamicText, StaticText};

bind_interrupts!(struct I2cIrqs {
    I2C1_IRQ => embassy_rp::i2c::InterruptHandler<embassy_rp::peripherals::I2C1>;
});

/// ever-increasing counter, used by the `DynamicText` example
static COUNT: AtomicU32 = AtomicU32::new(0);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // create display
    let sda = p.PIN_6;
    let scl = p.PIN_7;
    let mut config = embassy_rp::i2c::Config::default();
    config.frequency = 400_000;
    let bus = embassy_rp::i2c::I2c::new_async(p.I2C1, scl, sda, I2cIrqs, config);
    let interface = I2CDisplayInterface::new(bus);
    let display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    // create pages (need to be created before pagr
    // as resources are dropped in inverse order
    let mut screensaver = Screensaver::new(include_bytes!("./rust.bmp"));
    let mut text = StaticText::new("Hello, World!", &FONT_6X10);
    let mut dynamic_text = DynamicText::<_, 32, 1>::new(|| {
        let value = COUNT.load(Ordering::Relaxed);
        let mut output = String::new();
        uwrite!(output, "Uptime (s):\n{}", value).unwrap();

        output
    }, &FONT_6X10);
    spawner.must_spawn(count_up());
    let mut static_bmp = StaticImage::<Rgb565>::with_alignment(include_bytes!("./four_rings.bmp"), HorizontalAlignment::Right, VerticalAlignment::Bottom);
    let mut matrix_rain = DigitalRain::<21,7>::new(0xDA7A);

    // create pagrs object
    let mut pagrs = PageRotator::<5, _, _, _>::new(display);
    pagrs.init().await.unwrap();
    let _ = pagrs.add_page(&mut text);
    let _ = pagrs.add_page(&mut screensaver);
    let _ = pagrs.add_page(&mut dynamic_text);
    let _ = pagrs.add_page_with_duration(&mut static_bmp, Duration::from_secs(1));
    let _ = pagrs.add_page_with_duration(&mut matrix_rain, Duration::from_secs(10));

    // rotate contains an endless loop for refreshing display
    pagrs.rotate(spawner).await;
}

/// individual task, continuously increasing a counter every 5 seconds.
/// sets the value the `DynamicText` page displays
#[embassy_executor::task]
async fn count_up() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
        let value = COUNT.load(Ordering::Relaxed);
        COUNT.store(value + 1, Ordering::Relaxed);
    }
}
