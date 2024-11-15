use display_interface::DisplayError;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::BinaryColor;
use pagrs_core::Page;

pub struct StaticText<'a> {
    text: Text<'a, MonoTextStyle<'a, BinaryColor>>,
}

impl<'a> StaticText<'a> {
    pub const fn new(text: &'a str, font: &'a MonoFont<'a>) -> Self {
        let style = MonoTextStyle::new(font, BinaryColor::On);

        Self {
            text: Text::new(text, Point::new(0, font.character_size.height as i32), style)
        }
    }
}

impl<'a, D: DrawTarget<Color = BinaryColor, Error = DisplayError>> Page<D> for StaticText<'a> {
    fn render(&mut self, display: &mut D) -> Result<(), DisplayError> {
        self.text.draw(display)?;

        Ok(())
    }

    fn frames_per_second(&self) -> u8 {
        1
    }
}
