pagrs-screensaver: display a moving image as a page
====================================================

show a bmp image, moving from side to side across the display.
The image must be provided as bytes in RGB 565 encoding.

getting started
------------------

### minimum example
```rust
use pagrs_screensaver::{Screensaver};

async fn main() {
    let mut screensaver = Screensaver::new(include_bytes!("./rust.bmp"));
}
```
