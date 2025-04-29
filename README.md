pagrs: cycling content on embedded displays
====================================================

pagrs is a set of Rust libraries allowing you to cycle the content shown on an embedded display (currently limited to SSD 1306) for microcontroller environments. 
You define the individual pages and their duration and then the rotation starts, with you still in control (moving forward, moving backward).

First and foremost it is a **proof of concept** and was a learning experience for me, to get to know how to drive embedded graphics via Rust.  
But as it might be maybe of interest for some I decided to put it out there on Github.


getting started
-----------------

Example uses an SSD 1306 display and a raspberry pico:
```rust
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // get hardware periphery for display
    let sda = p.PIN_6;
    let scl = p.PIN_7;
    let i2c_channel = p.I2C1;

    // rotate contains an endless loop for refreshing display
    spawner.must_spawn(rotate(sda, scl, i2c_channel));

    // optional: a page controller sends controlling commands to the rotator
    let controller = PageController::new();

    loop {
        // optionally: control the display via `controller` struct
        /* controller.previous().await; */
        /* controller.next().await; */
    }
}

#[embassy_executor::task]
async fn rotate(sda: embassy_rp::peripherals::PIN_6, scl: embassy_rp::peripherals::PIN_7, i2c_channel: embassy_rp::peripherals::I2C1) {
    let spawner = Spawner::for_current_executor().await;

    // create display
    let mut config = embassy_rp::i2c::Config::default();
    config.frequency = 400_000;
    let bus = embassy_rp::i2c::I2c::new_async(i2c_channel, scl, sda, I2cIrqs, config);
    let interface = I2CDisplayInterface::new(bus);
    let display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    /* [...setup pages] */

    // create main object: "page rotator" with the necessary capacity of pages as type parameter
    let mut pagr = PageRotator::<5, _, _, _>::new(display);
    pagr.init().await.unwrap();
    /* [...add pages] */

    // start endless rotation loop:
    pagr.rotate(spawner).await
}
```

Each page first needs to be setup, according to the necessities of the page. See their corresponding crates & documentation.
Once its setup and available it can be registered to the `PageRotator` object. E.g. for a static text:

```rust
async fn rotate(...) {
    /* [...] */
    let mut text = StaticText::new("Hello, World!", &FONT_6X10);

    let mut pagr = PageRotator::<5, _, _, _>::new(display);
    pagr.init().await.unwrap();
    
    let _ = pagr.add_page(&mut text);
    /* [...] */
}
```

When registering a page the duration of the page can also be changed from the default:
```rust
async fn rotate(...) {
    /* [...] */
    let _ = pagr.add_page_with_duration(&mut text, Duration::from_secs(10));
    /* [...] */
}
```


structure
----------

- `pagrs`: an umbrella crate pulling in all other, necessary crates as dependencies.x Optional crates are controlled via features. Demo crates are not included.
- `pagrs-core`: page rotation and controlling mechanic as well as trait definitions. The main working horse.
- `pagrs-bmp`: page implementing displaying a static image in configured position
- `pagrs-matrix`: example page implementing "digital rain" as made famous by the movie Matrix
- `pagrs-screensaver`: page implementing a screensaver by displaying a moving image
- `pagrs-text`: pages implementing displaying a static or dynamically changing text
- `pagrs-demo-ssd1306`: fully working example project showcasing all pages using a Raspberry Pico microcontroller and an SSD1306 display


lifecycle of a page
---------------------

- each page has to be created before registering to the pagrs controller and need to stay alive for the whole duration of the application.
- everytime a page is rotated in the `activated()` method is called. 
- as long as the page is active the `render()` method is called for each frame.
- everytime a page is rotated out the `deactivated()` method is called.


write your own page
---------------------

see the documentation for `pagrs-core` to see more on how to write your own page. Also check existing pages as reference.
