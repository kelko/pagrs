use display_interface::DisplayError;
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::BinaryColor;
use heapless::String;
use pagrs_core::Page;

#[derive(Debug)]
pub struct DynamicText<'a, F, const LENGTH: usize = 64, const FRAMES_PER_SECOND: u8 = 24>
where F: Fn() -> String<LENGTH> {
    query_text: F,
    font: &'a MonoFont<'a>
}

impl<'a, F, const LENGTH: usize, const FRAMES_PER_SECOND: u8> DynamicText<'a, F, LENGTH, FRAMES_PER_SECOND>
where F: Fn() -> String<LENGTH> {
    pub fn new(query_text: F, font: &'a MonoFont<'a>) -> Self{
        Self{
            query_text,
            font,
        }
    }
}

impl<'a, F, const LENGTH: usize, const FRAMES_PER_SECOND: u8, D: DrawTarget<Color = BinaryColor, Error = DisplayError>> Page<D> for DynamicText<'a, F, LENGTH, FRAMES_PER_SECOND>
where F: Fn() -> String<LENGTH> {
    fn render(&mut self, display: &mut D) -> Result<(), DisplayError> {
        let content = &self.query_text;
        let content = content();
        let style = MonoTextStyle::new(self.font, BinaryColor::On);
        let text = Text::new(content.as_str(), Point::new(0, self.font.character_size.height as i32), style);
        text.draw(display)?;

        Ok(())
    }

    fn frames_per_second(&self) -> u8 {
        FRAMES_PER_SECOND
    }
}
