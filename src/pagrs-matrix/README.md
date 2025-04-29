pagrs-matrix: "digital rain" as page
====================================================

creates a random, digital rain as made famous by the movie Matrix.
Each glyph (included spacing) has 8 pixels width and 9 pixels height and can have one of 27 different styles

getting started
------------------

### minimum example

the dimension (rows & columns) as well as how many "workers" populate new glyphs on the screen 
need to be passed as type parameters

```rust
use pagrs_matrix::{DigitalRain};

async fn main() {
    // the seed for the randomizer needs to be provided.
    // the rows & columns (has to fit on the display) are configured as type parameters
    let mut matrix_rain = DigitalRain::<16, 7, 16>::new(0xDA7A);
}
```
