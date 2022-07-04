# EEI VFD

This library contains a driver for VFD screen from EEI tech.

It uses the [embedded graphics](https://crates.io/crates/embedded-graphics) library for the optional graphics support.

## Examples

There are multiple examples in the examples folder. Use `cargo run --example example_name` to try them.

```Rust
// Setup the epd
let mut epd = Epd4in2::new( & mut spi, cs, busy, dc, rst, & mut delay) ?;

// Setup the graphics
let mut display = Display4in2::default ();

// Draw some text
display.draw(
let _ = Text::new("Hello Rust!", Point::new(x, y))
.into_styled(text_style!(
            font = Font12x16,
            text_color = Black,
            background_color = White
        ))
.draw(display);
);

// Transfer the frame data to the epd and display it
epd.update_and_display_frame( & mut spi, & display.buffer()) ?;
```
