pagrs-bmp: display a static image as a page
============================================

show a fixed, static image. The image must be provided as bytes, encoded in a color-space supported by `tinybmp::Bmp`

getting started
------------------

### minimum example
```rust
use embedded_graphics::pixelcolor::Rgb565;
use pagrs_bmp::{StaticImage};

async fn main() {
    let mut static_bmp = StaticImage::<Rgb565>::new(
        include_bytes!("./four_rings.bmp"),
        HorizontalAlignment::Right,
        VerticalAlignment::Bottom,
    );
}
```

By default the image is aligned center (both horizontally and vertically). 

### aligned image

```rust
use embedded_graphics::pixelcolor::Rgb565;
use pagrs_bmp::{HorizontalAlignment, StaticImage, VerticalAlignment};

async fn main() {
    let mut static_bmp = StaticImage::<Rgb565>::with_alignment(
        include_bytes!("./four_rings.bmp")
    );
}
```

The placement of the image is controlled by the enum values passed to the factory method:
```rust
pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}
```
