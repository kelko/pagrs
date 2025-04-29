use display_interface::AsyncWriteOnlyDataCommand;
use embassy_time::{Duration, Timer};
use embedded_graphics::primitives::{Polyline, Primitive, PrimitiveStyle};
use embedded_graphics_core::geometry::{Dimensions, Point, Size};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::primitives::Rectangle;
use embedded_graphics_core::Drawable;
use embedded_layout::align::{horizontal, vertical, Align};
use embedded_layout::layout::linear::{FixedMargin, LinearLayout};
use embedded_layout::object_chain::Chain;
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::size::DisplaySizeAsync;
use ssd1306::Ssd1306Async;

static BRACKET_POINTS_LEFT: [Point; 4] = [
    Point::new(2, 0),
    Point::new(0, 0),
    Point::new(0, 15),
    Point::new(2, 15),
];
static BRACKET_POINTS_RIGHT: [Point; 4] = [
    Point::new(0, 0),
    Point::new(2, 0),
    Point::new(2, 15),
    Point::new(0, 15),
];

/// shows an icon for `pagrs`,
/// used as short splashscreen by the [PageRotator](crate::PageRotator) before starting to
/// rotate the actual [Pages](crate::Page)
pub(crate) struct SplashScreen<'a, DI, SIZE, MODE> {
    display: &'a mut Ssd1306Async<DI, SIZE, MODE>,
}

impl<'a, DI, SIZE> SplashScreen<'a, DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
{
    pub(crate) fn new(
        display: &'a mut Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>,
    ) -> Self {
        Self { display }
    }

    pub(crate) async fn show(self) {
        self.display.clear_buffer();
        self.display.flush().await.unwrap();

        let display_area = self.display.bounding_box();

        let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
        let thick_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 3);

        // Primitives to be displayed
        let center = Rectangle::new(Point::new(0, 0), Size::new(18, 18)).into_styled(thick_stroke);

        let left_bracket = Polyline::new(&BRACKET_POINTS_LEFT).into_styled(thin_stroke);
        let right_bracket = Polyline::new(&BRACKET_POINTS_RIGHT).into_styled(thin_stroke);

        // The layout
        LinearLayout::horizontal(
            Chain::new(left_bracket)
                .append(left_bracket)
                .append(center)
                .append(right_bracket)
                .append(right_bracket),
        )
            .with_alignment(vertical::Center)
            .with_spacing(FixedMargin(1))
            .arrange()
            .align_to(&display_area, horizontal::Center, vertical::Center)
            .draw(self.display)
            .unwrap();
        self.display.flush().await.unwrap();

        Timer::after(Duration::from_millis(500)).await;
    }
}
