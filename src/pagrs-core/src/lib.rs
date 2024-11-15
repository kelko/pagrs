#![no_std]

use display_interface::DisplayError;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::pixelcolor::BinaryColor;

mod splash_screen;
mod page_wrapper;
mod rotation;

const DEFAULT_FRAMES_PER_SECOND: u8 = 24;

pub trait Page<D: DrawTarget<Color = BinaryColor, Error = DisplayError>> {
    fn activated(&mut self) -> Result<(), D::Error> {
        Ok(())
    }
    fn render(&mut self, display: &mut D) -> Result<(), D::Error>;
    fn deactivated(&mut self)  -> Result<(), D::Error> {
        Ok(())
    }

    fn frames_per_second(&self) -> u8  {
        DEFAULT_FRAMES_PER_SECOND
    }
}

pub use rotation::PageRotator;