use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::internal::event::ErrorEvent;

pub struct Kernel {
    pub tick: AtomicU64,
    pub running: AtomicBool
} impl Kernel {
    pub fn new() -> Self { Self {
        tick: AtomicU64::new(0),
        running: AtomicBool::new(true)
    } }

    pub fn init(&mut self) {
        log::info!("Kernel.init()")
    }

    pub fn tick(&mut self) {
        let secs = self.tick.load(Ordering::SeqCst) / 1000;

        if self.tick.load(Ordering::SeqCst) % 1000 == 0 {
            log::info!("About {}s have passed.", secs);
        }

        if secs >= 10 { self.running.store(false, Ordering::SeqCst); }
    }

    pub fn on_error(&mut self, event: ErrorEvent) {
        log::error!("Kernel.on_error({:?})", event);
    }

    pub fn halt(&mut self) {
        log::info!("Kernel.halt()")
    }
}