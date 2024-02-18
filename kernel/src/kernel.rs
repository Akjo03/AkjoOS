use core::sync::atomic::{AtomicBool, Ordering};
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    serial_logger: &'a mut SerialPortLogger,
    display_manager: DisplayManager<'a>,
    pub running: AtomicBool
} impl<'a> Kernel<'a> {
    pub fn new(
        serial_logger: &'a mut SerialPortLogger,
        display_manager: DisplayManager<'a>
    ) -> Self { Self {
        serial_logger,
        display_manager,
        running: AtomicBool::new(true)
    } }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::Font9x18B));

        self.serial_logger.log(format_args!(
            "Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()
        ), SerialLoggingLevel::Info);

        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, _) => {
                driver.write_string("Hello, world!");
            }, _ => {}
        }

        self.display_manager.draw_all();
    }

    pub fn tick(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn halt(&mut self) {}
}