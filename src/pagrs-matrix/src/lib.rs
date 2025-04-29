#![no_std]

use core::cmp::PartialEq;
use display_interface::DisplayError;
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::geometry::{Point, Size};
use embedded_graphics_core::pixelcolor::BinaryColor;
use embedded_graphics_core::primitives::Rectangle;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use static_cell::StaticCell;
use pagrs_core::Page;

pub const PIXEL_PER_GLYPH_HEIGHT: usize = 9;
pub const PIXEL_PER_GLYPH_WIDTH: usize = 8;

static RANDOM: StaticCell<SmallRng> = StaticCell::new();

#[derive(Debug, Copy, Clone, PartialEq)]
enum Mode {
    Adding(usize),
    Removing(usize),
    Done,
}

#[derive(Debug, Copy, Clone)]
struct Worker {
    column: usize,
    row: usize,
    length: usize,
    mode: Mode,
}

impl Worker {
    const fn empty() -> Self {
        Worker {
            column: 0,
            row: 0,
            length: 0,
            mode: Mode::Done,
        }
    }

    fn new(column: usize, row: usize, length: usize) -> Self {
        Worker {
            column,
            row,
            length,
            mode: Mode::Adding(0),
        }
    }
}

#[derive(Debug)]
pub struct DigitalRain<const COLUMNS: usize, const ROWS: usize, const WORKER_COUNT: usize> {
    columns: [[u8; ROWS]; COLUMNS],
    workers: [Worker; WORKER_COUNT],
    random: &'static mut SmallRng,
    columns_with_workers: u64
}

impl<const COLUMNS: usize, const ROWS: usize, const WORKER_COUNT: usize> DigitalRain<COLUMNS, ROWS, WORKER_COUNT> {
    pub fn new(seed: u64) -> Self {
        let random = RANDOM.init(SmallRng::seed_from_u64(seed));

        DigitalRain {
            columns: [[0; ROWS]; COLUMNS],
            workers: [Worker::empty(); WORKER_COUNT],
            random,
            columns_with_workers: 0
        }
    }

    fn initialize(&mut self) {
        for column in self.columns.iter_mut() {
            for row in column.iter_mut() {
                *row = 0;
            }
        }

        for worker in self.workers.iter_mut() {
            *worker = Worker::empty();
        }

        self.columns_with_workers = 0;
    }

    fn update_state(&mut self) {
        let worker_count = self.random.random_range(2..WORKER_COUNT/2);

        for _ in 0..worker_count {
            let index = self.random.random_range(0..WORKER_COUNT);
            let worker = &mut self.workers[index];

            match worker.mode {
                Mode::Adding(index) => {
                    let glyph = self.random.random_range(1..27);
                    self.columns[worker.column][worker.row + index] = glyph;

                    if index == worker.length - 1 {
                        worker.mode = Mode::Removing(0);
                    } else {
                        worker.mode = Mode::Adding(index + 1);
                    }
                }
                Mode::Removing(index) => {
                    self.columns[worker.column][worker.row + index] = 0;

                    if index == worker.length - 1 {
                        let bit_mask = 1_u64 << worker.column;
                        self.columns_with_workers &= !bit_mask;
                        worker.mode = Mode::Done;
                    } else {
                        worker.mode = Mode::Removing(index + 1);
                    }
                }
                Mode::Done => {
                    let column = loop {
                        let candidate = self.random.random_range(0..COLUMNS);
                        let bit_mask = 1_u64 << candidate;
                        if self.columns_with_workers & bit_mask == bit_mask {
                            continue;
                        }

                        break candidate;
                    };

                    let row = self.random.random_range(0..(ROWS - 3));
                    let max_length = ROWS - row;
                    let length = self.random.random_range(5..ROWS).clamp(3, max_length);

                    self.columns_with_workers |= 1_u64 << column;
                    *worker = Worker::new(column, row, length);
                }
            }
        }

    }

    fn render_state<D: DrawTarget<Color=BinaryColor, Error=DisplayError>>(&mut self, display: &mut D) -> Result<(), D::Error> {
        for column_index in 0..COLUMNS {
            for row_index in 0..ROWS {
                let value = self.columns[column_index][row_index];
                if value == 0 {
                    continue;
                }

                self.paint_glyph(display, column_index, row_index, value)?;
            }
        }

        Ok(())
    }

    fn paint_glyph<D: DrawTarget<Color=BinaryColor, Error=DisplayError>>(&self, display: &mut D, column: usize, row: usize, value: u8) -> Result<(), D::Error> {
        let mut value = value % 27;

        // using braille style glyphs: 2 columns of 3 points each. Each point is tested individually
        // not braille compatible!

        for dot_row in 0..3_usize {
            let pattern = value % 3;
            value = value / 3;

            // left
            match pattern {
                0 | 1 => {
                    let rectangle = Rectangle::new(
                        Point::new((column * PIXEL_PER_GLYPH_WIDTH) as i32,
                                   (row * PIXEL_PER_GLYPH_HEIGHT + (3 * dot_row)) as i32),
                        Size::new(2, 2),
                    );
                    display.fill_solid(&rectangle, BinaryColor::On)?;
                }
                _ => {}
            };

            // right
            match pattern {
                0 | 2 => {
                    let rectangle = Rectangle::new(
                        Point::new((column * PIXEL_PER_GLYPH_WIDTH + 3) as i32,
                                   (row * PIXEL_PER_GLYPH_HEIGHT + (3 * dot_row)) as i32),
                        Size::new(2, 2),
                    );
                    display.fill_solid(&rectangle, BinaryColor::On)?;
                }
                _ => {}
            };
        }

        Ok(())
    }
}

impl<const COLUMNS: usize, const ROWS: usize, const WORKER_COUNT: usize, D: DrawTarget<Color=BinaryColor, Error=DisplayError>> Page<D> for DigitalRain<COLUMNS, ROWS, WORKER_COUNT> {
    fn activated(&mut self) -> Result<(), DisplayError> {
        self.initialize();

        Ok(())
    }

    fn render(&mut self, display: &mut D) -> Result<(), DisplayError> {
        // adding & removing glyphs
        self.update_state();

        // paint current state
        self.render_state(display)?;

        Ok(())
    }

    fn frames_per_second(&self) -> u8 {
        8
    }
}
