use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::internal::event::{ErrorEvent, ErrorLevel};
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
        self.display_manager.set_mode(DisplayMode::Text(Fonts::Font10x20));

        self.serial_logger.log(&format_args!(
            "Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()
        ), SerialLoggingLevel::Info);
    }

    pub fn tick(&mut self) {
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

    pub fn on_error(&mut self, error_event: ErrorEvent) {
        match error_event.level() {
            ErrorLevel::Fault => {
                self.serial_logger.log(&format_args!(
                    "\nKernel encountered a fault: {}",
                    error_event.message(),
                ), SerialLoggingLevel::Warning);
            }, ErrorLevel::Abort => {
                crate::abort(&format!(
                    "\nKernel encountered an unrecoverable error: {}",
                    error_event.message()
                ), Some(&mut self.display_manager));
            }, _ => {}
        }
    }

    pub fn halt(&mut self) {}
}