#![no_std]

use display_interface::DisplayError;
use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::image::Image;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::{BinaryColor, Rgb565};
use tinybmp::Bmp;
use pagrs_core::Page;

pub struct Screensaver<'a> {
    bmp: Bmp<'a, Rgb565>,
    index: i32,
    left_to_right: bool,
}

impl<'a> Screensaver<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        let bmp = Bmp::from_slice(bytes).expect("Failed to load BMP image");
        Self {
            bmp,
            index: 0,
            left_to_right: true,
        }
    }
}

impl<D: DrawTarget<Color = BinaryColor, Error = DisplayError>> Page<D> for Screensaver<'_> {
    fn render(&mut self, display: &mut D) -> Result<(), DisplayError>{
        let im: Image<Bmp<Rgb565>> = Image::new(&self.bmp, Point::new(self.index, 0));
        im.draw(&mut display.color_converted())?;

        self.index = match self.index {
            0 => {
                self.left_to_right = true;
                1
            }
            64 => {
                self.left_to_right = false;
                63
            }
            val => {
                if self.left_to_right {
                    val + 1
                } else {
                    val - 1
                }
            }
        };

        Ok(())
    }
}
