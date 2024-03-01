use alloc::format;
use alloc::string::ToString;
use core::sync::atomic::Ordering;
use crate::api::event::{ErrorEvent, EventErrorLevel};
use crate::{KernelRuntime, Kernel};
use crate::api::display::Fonts;
use crate::drivers::display::DisplayDriverType;
use crate::managers::display::DisplayMode;

impl KernelRuntime for Kernel {
    fn init(&mut self) {
        self.display_manager.set_mode(DisplayMode::Text(Fonts::default()));
    }

    fn tick(&mut self) {
        let current_tick = self.tick.load(Ordering::SeqCst);

        match self.display_manager.get_driver() {
            DisplayDriverType::Text(driver, ..) => {
                driver.clear_buffer();
                driver.write_string(format!(
                    "Tick {} at {}",
                    current_tick, self.time_manager.with_clock(
                        |clock| clock.now().to_string()
                    ).unwrap_or("N/A".to_string())
                ).as_str());

                if current_tick % 500 == 0 {
                    driver.blink();
                }
            }, _ => {}
        }
        self.display_manager.draw_all();

        if current_tick >= 10000 {
            self.running.store(false, Ordering::SeqCst);
        }
    }

    fn on_error(&mut self, event: ErrorEvent) {
        match event.level() {
            EventErrorLevel::Fault => {
                log::error!("Kernel encountered a fault: {}", event.message());
            }, EventErrorLevel::Abort => {
                crate::abort(&format!(
                    "\n Kernel encountered an unrecoverable error: {}",
                    event.message()
                ), Some(&mut self.display_manager))
            }, _ => {}
        }
    }

    fn halt(&mut self) {}
}