use core::sync::atomic::Ordering;
use crate::internal::event::ErrorEvent;
use crate::{KernelRuntime, Kernel};

impl KernelRuntime for Kernel {
    fn init(&mut self) {}

    fn tick(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }

    fn on_error(&mut self, _event: ErrorEvent) {}

    fn halt(&mut self) {}
}