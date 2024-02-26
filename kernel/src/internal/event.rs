use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::mutex::Mutex;
use spin::Once;
use crate::internal::cmos::DateTime;

static EVENT_DISPATCHER: Once<EventDispatcher> = Once::new();

#[derive(Debug, Clone)]
pub enum Event {
    Timer,
    Rtc(DateTime),
    Error(ErrorEvent)
} impl Event {
    pub fn error(event: ErrorEvent) -> Self {
        Event::Error(event)
    }
}

#[derive(Debug, Clone)]
pub enum ErrorEvent {
    Breakpoint(String),
    InvalidOpcode(String),
    InvalidTss(String, u64),
    PageFault(String, u64),
    GeneralProtectionFault(String, u64),
    DoubleFault(String, u64)
}

pub trait EventHandler {
    fn handle(&mut self, event: Event);
}

pub struct EventDispatcher {
    handlers: Mutex<Vec<Arc<Mutex<dyn EventHandler + Send>>>>,
    queue: Mutex<VecDeque<Event>>,
    new_event: AtomicBool
} #[allow(dead_code)] impl EventDispatcher {
    pub fn global() -> &'static Self {
        EVENT_DISPATCHER.call_once(|| EventDispatcher::new())
    }

    fn new() -> Self { Self {
        handlers: Mutex::new(Vec::new()),
        queue: Mutex::new(VecDeque::new()),
        new_event: AtomicBool::new(false)
    } }

    pub fn register(&self, handler: Arc<Mutex<dyn EventHandler + Send>>) {
        self.handlers.lock().push(handler);
    }

    pub fn push(&self, event: Event) {
        self.queue.lock().push_back(event);
        self.new_event.store(true, Ordering::Relaxed)
    }

    pub fn dispatch(&self) {
        crate::internal::idt::without_interrupts(|| {
            let mut local_queue = VecDeque::new();

            core::mem::swap(&mut *self.queue.lock(), &mut local_queue);

            while let Some(event) = local_queue.pop_front() {
                let mut handlers = self.handlers.try_lock();
                if let Some(handlers) = handlers.as_mut() {
                    for handler in handlers.iter_mut() {
                        let mut handler = handler.try_lock();
                        if let Some(handler) = handler.as_mut() {
                            handler.handle(event.clone());
                        } else { log::warn!("Event handler is locked, skipping dispatch."); }
                    }
                } else { log::warn!("Event handlers are locked, skipping dispatch."); return; }
            }

            self.new_event.store(false, Ordering::Relaxed);
        })
    }
}