pagrs-text: display a simple text as a page
============================================

show either a fixed, static text (`StaticText` struct) or a text which is read on each refresh (`DynamicText` struct).

getting started
------------------

### static text

Provide the text as well as the font & size statically in the factory method:
```rust
use embedded_graphics::mono_font::ascii::FONT_6X10;
use pagrs_text::StaticText;

async fn main() {
    let mut text = StaticText::new("Hello, World!", &FONT_6X10);
    // use text page
}
```

### dynamic text

Provide the font & size statically in the factory method and provide a callback to generate the text content:
```rust
use embedded_graphics::mono_font::ascii::FONT_6X10;
use pagrs_text::StaticText;

async fn main() {
    let mut dynamic_text = DynamicText::<_, 32, 1>::new(
        || {
            // COUNT being some external value, that might be changing
            let value = COUNT.load(Ordering::Relaxed);
            let mut output = String::new();
            uwrite!(output, "Uptime (s):\n{}", value).unwrap();

            output
        },
        &FONT_6X10,
    );
    // use text page
}
```

The maximum length of the string has to be passed as one of the generic parameters (32 in the example above). The second parameter controls the frame rate.
In the example above it's set to 1fps.
