use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer, WithTimeout};
use heapless::Vec;
use ssd1306::mode::{BufferedGraphicsModeAsync, DisplayConfigAsync};
use ssd1306::size::DisplaySizeAsync;
use ssd1306::{Ssd1306Async};
use crate::Page;
use crate::page_wrapper::PageWrapper;
use crate::splash_screen::SplashScreen;

static CANCEL_PAGE: AtomicBool = AtomicBool::new(false);
static ITERATION_INDEX: AtomicU8 = AtomicU8::new(0);
static CONTROL_CODE: AtomicU8 = AtomicU8::new(0);

const CONTROL_CODE_NOTHING: u8 = 0;
const CONTROL_CODE_NEXT: u8 = 1;
const CONTROL_CODE_PREVIOUS: u8 = 2;

pub struct PageRotator<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> {
    pages: Vec<PageWrapper<'a, DI, SIZE, MODE>, PAGE_COUNT>,
    display: RefCell<Ssd1306Async<DI, SIZE, MODE>>,
}

impl<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> PageRotator<'a, PAGE_COUNT, DI, SIZE, MODE> {
    pub fn new(display: Ssd1306Async<DI, SIZE, MODE>) -> Self {
        Self {
            pages: Vec::new(),
            display: RefCell::new(display),
        }
    }
}

impl<'a, const PAGE_COUNT: usize, DI, SIZE> PageRotator<'a, PAGE_COUNT, DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
{
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        let mut display = self.display.borrow_mut();
        display.init().await?;
        SplashScreen::new(&mut display).show().await;

        Ok(())
    }

    pub async fn rotate(&self, spawner: Spawner) -> ! {
        let mut index = 0;
        let page_count = self.pages.len();

        loop {
            let iter_index = (ITERATION_INDEX.load(Ordering::Acquire) + 1) % 255;
            ITERATION_INDEX.store(iter_index, Ordering::Release);

            CANCEL_PAGE.store(false, Ordering::Release);

            let control_code = CONTROL_CODE.load(Ordering::Acquire);
            CONTROL_CODE.store(CONTROL_CODE_NOTHING, Ordering::Release);
            match control_code {
                CONTROL_CODE_PREVIOUS => {
                    index = (index + page_count - 2) % page_count;
                },
                _ => {},
            }

            let page = self.pages.get(index).unwrap();
            index = (index + 1) % page_count;

            let duration = page.custom_duration.unwrap_or(Duration::from_secs(5));
            spawner.must_spawn(next_pane_counter(duration, iter_index));

            page
                .take_over(&mut self.display.borrow_mut(), || {
                    if CANCEL_PAGE.load(Ordering::Acquire) {
                        return true;
                    }

                    let code = CONTROL_CODE.load(Ordering::Acquire);
                    code != CONTROL_CODE_NOTHING
                })
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

    pub fn controller(&self) -> PageController {
        PageController {}
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn next_pane_counter(time: Duration, my_index: u8) {
    Timer::after(time).await;

    let current_index = ITERATION_INDEX.load(Ordering::Acquire);
    if my_index != current_index {
        return
    }

    CANCEL_PAGE.store(true, Ordering::Release);
}

pub struct PageController {}

impl PageController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn next(&self) {
        CONTROL_CODE.store(CONTROL_CODE_NEXT, Ordering::Release);
    }

    pub fn previous(&self) {
        CONTROL_CODE.store(CONTROL_CODE_PREVIOUS, Ordering::Release);
    }
}
