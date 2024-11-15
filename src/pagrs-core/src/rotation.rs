use core::sync::atomic::{AtomicBool, Ordering};
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use heapless::Vec;
use ssd1306::mode::{BufferedGraphicsModeAsync, DisplayConfigAsync};
use ssd1306::size::DisplaySizeAsync;
use ssd1306::{Ssd1306Async};
use crate::Page;
use crate::page_wrapper::PageWrapper;
use crate::splash_screen::SplashScreen;

static NEXT_PANE: AtomicBool = AtomicBool::new(false);

pub struct PageRotator<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> {
    pages: Vec<PageWrapper<'a, DI, SIZE, MODE>, PAGE_COUNT>,
    display: Ssd1306Async<DI, SIZE, MODE>,
}

impl<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> PageRotator<'a, PAGE_COUNT, DI, SIZE, MODE> {
    pub fn new(display: Ssd1306Async<DI, SIZE, MODE>) -> Self {
        Self {
            pages: Vec::new(),
            display,
        }
    }
}

impl<'a, const PAGE_COUNT: usize, DI, SIZE> PageRotator<'a, PAGE_COUNT, DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
{
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        self.display.init().await?;
        SplashScreen::new(&mut self.display).show().await;

        Ok(())
    }

    pub async fn rotate(mut self, spawner: Spawner) -> ! {
        let mut index = 0;
        let page_count = self.pages.len();

        loop {
            NEXT_PANE.store(false, Ordering::Relaxed);
            let page = self.pages.get_mut(index).unwrap();
            index = (index + 1) % page_count;

            let duration = page.custom_duration.unwrap_or(Duration::from_secs(5));
            spawner.must_spawn(next_pane_counter(duration));

            page
                .take_over(&mut self.display, || NEXT_PANE.load(Ordering::Relaxed))
                .await.unwrap();
        }
    }

    pub fn add_page<P: Page<Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>>>(&mut self, page: &'a mut P) -> Result<(), ()> {
        if let Err(_) = self.pages.push(PageWrapper::new(page)) {
            return Err(());
        }

        Ok(())
    }

    pub fn add_page_with_duration<P: Page<Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>>>(&mut self, page: &'a mut P, duration: Duration) -> Result<(), ()> {
        if let Err(_) = self.pages.push(PageWrapper::with_custom_duration(page, duration)) {
            return Err(());
        }

        Ok(())
    }
}

#[embassy_executor::task]
async fn next_pane_counter(time: Duration) {
    Timer::after(time).await;
    NEXT_PANE.store(true, Ordering::Relaxed);
}
