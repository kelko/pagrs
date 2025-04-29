#![no_std]

use display_interface::DisplayError;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::pixelcolor::BinaryColor;

mod splash_screen;
mod page_wrapper;
mod rotation;

/// the default framerate, if not overwritten by a page.
/// Uses the value used traditionally by most movies
const DEFAULT_FRAMES_PER_SECOND: u8 = 24;

/// Definition of a page that can be rotated in to be shown by the [PageRotator].
///
/// implementing structs MUST provide the [`render`](Page::render) method, while for the other methods default implementations exist.
///
/// ## life cycle
///
/// - each page has to be created before registering to the [PageRotator] and need to stay alive for the whole duration of the application.
/// - everytime a page is rotated in the [`activated`](Page::activated) method is called.
/// - as long as the page is active the [`render`](Page::render) method is called for each frame.
/// - everytime a page is rotated out the [`deactivated`](Page::deactivated) method is called.
pub trait Page<D: DrawTarget<Color = BinaryColor, Error = DisplayError>> {
    /// inform the page, that it is rotated in and will be visible on the display
    /// and should prepare internal state so it can be [`render`](Page::render)-ed.
    fn activated(&mut self) -> Result<(), D::Error> {
        Ok(())
    }

    /// draw the content of the page on the provided `display`.
    /// The method is called as often per second as [`Page::frames_per_second`] defines
    ///
    /// parameter:
    /// `display`: a [DrawTarget] object that can be used to draw something on the display
    fn render(&mut self, display: &mut D) -> Result<(), D::Error>;

    /// inform the page, that it is rotated out of being visible on the display.
    /// e.g. it can now free some additional resources or reset state
    fn deactivated(&mut self)  -> Result<(), D::Error> {
        Ok(())
    }

    /// return the required framerate of that page.
    /// the value returned is how often the [`render`](Page::render) method will be called per second.
    fn frames_per_second(&self) -> u8  {
        DEFAULT_FRAMES_PER_SECOND
    }
}

pub use rotation::{PageRotator, PageController};