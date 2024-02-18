use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::internal::event::{ErrorEvent, ErrorLevel, Event, EventHandler, EventMask};
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    serial_logger: &'a mut SerialPortLogger,
    display_manager: DisplayManager<'a>,
    pub running: AtomicBool,
    pub tick: AtomicU64
} impl<'a> Kernel<'a> {
    pub fn new(
        serial_logger: &'a mut SerialPortLogger,
        display_manager: DisplayManager<'a>
    ) -> Self { Self {
        serial_logger,
        display_manager,
        running: AtomicBool::new(true),
        tick: AtomicU64::new(0)
    } }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::Font9x18B));

        self.serial_logger.log(&format_args!(
            "Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()
        ), SerialLoggingLevel::Info);
    }

    fn tick(&mut self) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, _) => {
                driver.clear_buffer();
                driver.write_string(format!(
                    "C:\\> Welcome to AkjoOS! Tick: {}",
                    self.tick.load(Ordering::Relaxed)
                ).as_str());
            }, _ => {}
        }

        self.display_manager.draw_all();

        if self.tick.load(Ordering::Relaxed) == 100 {
            self.running.store(false, Ordering::Relaxed);
        }
    }

    fn on_error(&mut self, error_event: ErrorEvent) {
        match error_event.level() {
            ErrorLevel::Fault => {

            }, ErrorLevel::Abort => {
                crate::abort(&format!(
                    "\nKernel encountered an unrecoverable error: {}",
                    error_event.message()
                ), Some(&mut self.display_manager));
            }, _ => {}
        }
    }

    pub fn halt(&mut self) {}
} impl EventHandler for Kernel<'_> {
    fn handle(&mut self, event: Event) {
        match event {
            Event::TimerInterrupt => {
                self.tick.fetch_add(1, Ordering::Relaxed);
                self.tick();
            }, Event::Error(event) => {
                self.on_error(event);
            }
        }
    }

    fn mask(&self) -> EventMask { EventMask::all() }
}