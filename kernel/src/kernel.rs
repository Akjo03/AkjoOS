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
        log::info!("Kernel.tick({})", self.tick.load(Ordering::SeqCst));
        if self.tick.load(Ordering::SeqCst) == 1 {
            self.running.store(false, Ordering::SeqCst); // Stop the kernel after the first tick
        }
    }

    pub fn on_error(&mut self, event: ErrorEvent) {
        log::error!("Kernel.on_error({:?})", event);
    }

    pub fn halt(&mut self) {
        log::info!("Kernel.halt()")
    }
}