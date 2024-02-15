use alloc::format;
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::managers::display::{DisplayManager, DisplayMode};

pub struct Kernel<'a> {
    serial_logger: &'a mut SerialPortLogger,
    display_manager: DisplayManager<'a>,
    pub running: bool
} impl<'a> Kernel<'a> {
    pub fn new(
        serial_logger: &'a mut SerialPortLogger,
        display_manager: DisplayManager<'a>
    ) -> Self { Self {
        serial_logger,
        display_manager,
        running: true
    } }

    pub fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::Font9x18B));

        self.serial_logger.log(format_args!(
            "Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()
        ), SerialLoggingLevel::Info);
    }

    pub fn tick(&mut self, tick: u64) {
        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, _) => {
                driver.clear_buffer();

                driver.write_string(format!(
                    "C:\\> Kernel at tick {}.",
                    tick
                ).as_str());
                if tick % 2800 == 0 { driver.blink(); }
            }, _ => panic!("Unsupported display driver type!")
        }

        self.display_manager.draw_all();

        if tick > 50000 { self.running = false; }
    }

    pub fn halt(&mut self) -> ! {
        loop {}
    }
}