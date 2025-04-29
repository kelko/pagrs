#![no_std]

use display_interface::DisplayError;
use embedded_graphics::draw_target::DrawTargetExt;
use embedded_graphics::image::Image;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::Drawable;
use embedded_graphics_core::geometry::Point;
use embedded_graphics_core::pixelcolor::{BinaryColor, PixelColor, Rgb555, Rgb565, Rgb888};
use embedded_layout::align::{horizontal, vertical, Align};
use tinybmp::Bmp;
use pagrs_core::Page;

#[derive(Debug, PartialEq)]
/// define the horizontal alignment of the image: centered, on the left or on the right?
pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

#[derive(Debug, PartialEq)]
/// define the vertical alignment of the image: centered, at the top or at the bottom?
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

#[derive(Debug)]
/// show a fixed, static image.
/// # examples
/// ## minimum code, centered
/// ```rust
/// # use embedded_graphics::pixelcolor::Rgb565;
/// use pagrs_bmp::{StaticImage};
///
///  let mut static_bmp = StaticImage::<Rgb565>::new(
///     include_bytes!("./four_rings.bmp")
///  );
/// ```
///
/// ## aligned image
///
/// ```rust
/// # use embedded_graphics::pixelcolor::Rgb565;
/// use pagrs_bmp::{HorizontalAlignment, StaticImage, VerticalAlignment};
///
///  let mut static_bmp = StaticImage::<Rgb565>::with_alignment(
///     include_bytes!("./four_rings.bmp"),
///     HorizontalAlignment::Right,
///     VerticalAlignment::Bottom,
///  );
// ```
pub struct StaticImage<'a, C> {
    bmp: Bmp<'a, C>,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl<'a, C> StaticImage<'a, C>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888>,
{
    /// create a new page showcasing the image provided as bytes (must be encoded so `tinybmp` understands it).
    /// The image will be centered vertically and horizontally
    pub fn new(bytes: &'a [u8]) -> Self {
        let bmp = Bmp::from_slice(bytes).expect("Failed to load BMP image");
        Self {
            bmp,
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
        }
    }

    /// create a new page showcasing the image provided as bytes (must be encoded so `tinybmp` understands it).
    /// the image will be shown according to the passed alignment values
    pub fn with_alignment(bytes: &'a [u8], horizontal_alignment: HorizontalAlignment, vertical_alignment: VerticalAlignment) -> Self {
        let bmp = Bmp::from_slice(bytes).expect("Failed to load BMP image");
        Self {
            bmp,
            horizontal_alignment,
            vertical_alignment,
        }
    }
}

impl<'a, C, D> Page<D> for StaticImage<'a, C>
where
    C: PixelColor + From<Rgb555> + From<Rgb565> + From<Rgb888> + Into<BinaryColor>,
    D: DrawTarget<Color=BinaryColor, Error=DisplayError>,
{
    fn render(&mut self, display: &mut D) -> Result<(), DisplayError> {
        let display_area = display.bounding_box();
        let mut img = Image::new(&self.bmp, Point::new(0, 0));

        match (&self.horizontal_alignment, &self.vertical_alignment) {
            (HorizontalAlignment::Center, VerticalAlignment::Center) => {
                img = img.align_to(&display_area, horizontal::Center, vertical::Center);
            },
            (HorizontalAlignment::Left, VerticalAlignment::Center) => {
                img = img.align_to(&display_area, horizontal::Left, vertical::Center);
            },
            (HorizontalAlignment::Right, VerticalAlignment::Center) => {
                img = img.align_to(&display_area, horizontal::Right, vertical::Center);
            },

            (HorizontalAlignment::Center, VerticalAlignment::Top) => {
                img = img.align_to(&display_area, horizontal::Center, vertical::Top);
            },
            (HorizontalAlignment::Left, VerticalAlignment::Top) => {
                img = img.align_to(&display_area, horizontal::Left, vertical::Top);
            },
            (HorizontalAlignment::Right, VerticalAlignment::Top) => {
                img = img.align_to(&display_area, horizontal::Right, vertical::Top);
            },

            (HorizontalAlignment::Center, VerticalAlignment::Bottom) => {
                img = img.align_to(&display_area, horizontal::Center, vertical::Bottom);
            },
            (HorizontalAlignment::Left, VerticalAlignment::Bottom) => {
                img = img.align_to(&display_area, horizontal::Left, vertical::Bottom);
            },
            (HorizontalAlignment::Right, VerticalAlignment::Bottom) => {
                img = img.align_to(&display_area, horizontal::Right, vertical::Bottom);
            },
        }

        img.draw(&mut display.color_converted())?;

        Ok(())
    }

    fn frames_per_second(&self) -> u8 {
        1
    }
}
