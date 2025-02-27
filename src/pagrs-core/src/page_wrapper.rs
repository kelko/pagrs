use core::cell::RefCell;
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use embassy_time::{Duration, Ticker};
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::size::DisplaySizeAsync;
use ssd1306::Ssd1306Async;
use crate::{Page, DEFAULT_FRAMES_PER_SECOND};

pub(crate) struct PageWrapper<'a, DI, SIZE, MODE> {
    page: RefCell<&'a mut dyn Page<Ssd1306Async<DI, SIZE, MODE>>>,
    pub custom_duration: Option<Duration>
}

impl<'a, DI, SIZE, MODE> PageWrapper<'a, DI, SIZE, MODE> {
    pub(crate) fn new(page: &'a mut dyn Page<Ssd1306Async<DI, SIZE, MODE>>) -> Self {
        Self {
            page: RefCell::new(page),
            custom_duration: None
        }
    }

    pub(crate) fn with_custom_duration(page: &'a mut dyn Page<Ssd1306Async<DI, SIZE, MODE>>, duration: Duration) -> Self {
        Self {
            page: RefCell::new(page),
            custom_duration: Some(duration)
        }
    }
}

impl<'a, DI, SIZE> PageWrapper<'a, DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
{
    pub(crate) async fn take_over<F: Fn() -> bool> (
        &self,
        display: &mut Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>,
        cancel: F,
    ) -> Result<(), DisplayError> {
        let mut page = self.page.borrow_mut();
        let mut frames_per_second = page.frames_per_second();
        // a value of 0 for frames per second can't be allowed. resetting it to a default
        if frames_per_second == 0 {
            frames_per_second = DEFAULT_FRAMES_PER_SECOND;
        }
        let mut ticker = Ticker::every(Duration::from_millis(1000 / frames_per_second as u64));

        page.activated()?;
        loop {
            display.clear_buffer();
            page.render(display)?;
            display.flush().await?;

            let shall_cancel = cancel();
            if shall_cancel {
                break;
            }

            ticker.next().await;
        }
        page.deactivated()?;

        Ok(())
    }
}
