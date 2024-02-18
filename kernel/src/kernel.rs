use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::api::display::Fonts;
use crate::internal::event::{Event, EventHandler, EventMask};
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

        self.serial_logger.log(format_args!(
            "Kernel told display manager to use display mode {}.",
            self.display_manager.get_display_mode()
        ), SerialLoggingLevel::Info);
    }

    fn tick(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn halt(&mut self) {}
} impl EventHandler for Kernel<'_> {
    fn handle(&mut self, event: Event) {
        match event {
            Event::TimerInterrupt => {
                self.tick.fetch_add(1, Ordering::Relaxed);
                self.tick();
            }, _ => {}
        }
    }

    fn mask(&self) -> EventMask { EventMask::all() }
}