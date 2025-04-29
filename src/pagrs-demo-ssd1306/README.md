pagrs demo: SSD 1306 + RasPi Pico
====================================

showcases the pagrs library using all default pages types.

configuration
--------------

page rotation:

- static text: "Hello, World"
- screensaver with the Rust logo
- dynamic text showing the uptime in seconds (calculated by a parallel executed task)
- static bmp: four circles in the lower right corner (for 1 second)
- digital rain (for 10 seconds)

additionally, in the "main" loop, the page controller sends at random times either a "next" or "previous" command


hardware requirements
------------------------

- Raspberry Pico (RP2040) microcontroller
- SSD 1306 display
