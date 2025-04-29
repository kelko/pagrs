use core::cell::{RefCell};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use display_interface::{AsyncWriteOnlyDataCommand, DisplayError};
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver};
use embassy_time::{Duration, Timer};
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
static mut CHANNEL: MaybeUninit<Channel<NoopRawMutex, u8, 1>> = MaybeUninit::uninit();

const CONTROL_CODE_NOTHING: u8 = 0;
const CONTROL_CODE_NEXT: u8 = 1;
const CONTROL_CODE_PREVIOUS: u8 = 2;

/// the main actor of `pagrs`.
///
/// It keeps a mut reference to all pages and owns the embedded display and decides which
/// page is active at any given time, rotating through them.
///
/// **infos**:
/// - it currently only works for with a [Ssd1306Async] as display.
/// - it reserves the memory for the page vec ahead of time, statically, as to not need any `alloc`.
/// - before it can rotate first the `init` method must be called
///
/// ## type parameters
/// - `PAGE_COUNT`: the maximum amount of [pages](crate::Page) that can be registered to the rotator
/// - `DI`, `SIZE`, `MODE`: type parameters that are passed 1:1 to the [Ssd1306Async] display
pub struct PageRotator<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> {
    pages: Vec<PageWrapper<'a, DI, SIZE, MODE>, PAGE_COUNT>,
    display: RefCell<Ssd1306Async<DI, SIZE, MODE>>,
}

impl<'a, const PAGE_COUNT: usize, DI, SIZE, MODE> PageRotator<'a, PAGE_COUNT, DI, SIZE, MODE> {
    /// create a new [PageRotator] and reserves the memory for the page vector.
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
    /// initialize the display
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        let mut display = self.display.borrow_mut();
        display.init().await?;
        SplashScreen::new(&mut display).show().await;

        Ok(())
    }

    /// start the page rotation, cycling through all registered [pages](crate::Page).
    /// This method never returns. All pages must be registered before calling this method.
    pub async fn rotate(&self, spawner: Spawner) -> ! {
        let mut index = 0;
        let page_count = self.pages.len();

        let channel = unsafe { CHANNEL.write(Channel::new()) };

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
            spawner.must_spawn(next_pane_counter(duration, channel.receiver(), iter_index));

            page
                .take_over(&mut self.display.borrow_mut(), || {
                    CANCEL_PAGE.load(Ordering::Acquire)
                })
                .await.unwrap();
        }
    }

    /// register a new [Page] with the default duration for the page rotation.
    pub fn add_page<P: Page<Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>>>(&mut self, page: &'a mut P) -> Result<(), ()> {
        if let Err(_) = self.pages.push(PageWrapper::new(page)) {
            return Err(());
        }

        Ok(())
    }

    /// register a new [Page] with a custom duration for the page rotation
    pub fn add_page_with_duration<P: Page<Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>>>(&mut self, page: &'a mut P, duration: Duration) -> Result<(), ()> {
        if let Err(_) = self.pages.push(PageWrapper::with_custom_duration(page, duration)) {
            return Err(());
        }

        Ok(())
    }

    /// create a [PageController] for the [PageRotator] to be able to control aspects of it after rotation starts.
    pub fn controller(&self) -> PageController {
        PageController {}
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn next_pane_counter(time: Duration, rec: Receiver<'static, NoopRawMutex, u8, 1>, my_index: u8) {
    select(Timer::after(time), rec.receive()).await;

    let current_index = ITERATION_INDEX.load(Ordering::Acquire);
    if my_index != current_index {
        return
    }

    CANCEL_PAGE.store(true, Ordering::Release);
}

/// control the [PageRotator] after it starts rotating by sending commands to it via this
/// [PageController].
pub struct PageController {}

impl PageController {
    pub fn new() -> Self {
        Self {}
    }

    /// instruct the [PageRotator] to cycle now to the next page
    pub async fn next(&self) {
        CONTROL_CODE.store(CONTROL_CODE_NEXT, Ordering::Release);
        let channel = unsafe { CHANNEL.assume_init_ref() };
        channel.send(0).await;
    }

    /// instruct the [PageRotator] to cycle now to the previous page
    pub async fn previous(&self) {
        CONTROL_CODE.store(CONTROL_CODE_PREVIOUS, Ordering::Release);
        let channel = unsafe { CHANNEL.assume_init_ref() };
        channel.send(0).await;
    }
}
