use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::mutex::Mutex;
use spin::Once;
use crate::internal::cmos::Rtc;

static EVENT_DISPATCHER: Once<EventDispatcher> = Once::new();

#[derive(Debug, Clone)]
pub enum Event {
    /// A timer event is triggered when the system timer ticks.
    Timer,
    /// A real-time clock event is triggered when the real-time clock ticks.
    Rtc(Rtc),
    /// An error event is triggered when the kernel encounters an error.
    Error(ErrorEvent)
} impl Event {
    pub fn error(event: ErrorEvent) -> Self {
        Event::Error(event)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum EventErrorLevel {
    /// A (non-maskable) interrupt is an exception that cannot be ignored or masked and
    /// is a result of a serious hardware error.
    Interrupt,
    /// A trap is an exception that just gets reported and then the program continues.
    Trap,
    /// A fault is an exception that must be handled and can be recovered from.
    Fault,
    /// An abort is an exception that cannot be recovered from and the program must be terminated.
    Abort,
}

#[derive(Debug, Clone)]
pub enum ErrorEvent {
    /// A breakpoint was encountered.
    Breakpoint(String),
    /// An invalid opcode was encountered.
    InvalidOpcode(String),
    /// An invalid Task State Segment was encountered.
    InvalidTss(String, u64),
    /// A page fault was encountered.
    PageFault(String, u64),
    /// A general protection fault was encountered.
    GeneralProtectionFault(String, u64),
    /// A double fault was encountered.
    DoubleFault(String, u64)
} #[allow(dead_code)] impl ErrorEvent {
    /// Returns the message associated with the error event.
    pub fn message(&self) -> &String {
        match self {
            ErrorEvent::Breakpoint(message) => message,
            ErrorEvent::InvalidOpcode(message) => message,
            ErrorEvent::InvalidTss(message, ..) => message,
            ErrorEvent::PageFault(message, ..) => message,
            ErrorEvent::GeneralProtectionFault(message, ..) => message,
            ErrorEvent::DoubleFault(message, ..) => message
        }
    }

    /// Returns the level of the error event.
    pub fn level(&self) -> EventErrorLevel {
        match self {
            ErrorEvent::Breakpoint(..) => EventErrorLevel::Trap,
            ErrorEvent::InvalidOpcode(..) => EventErrorLevel::Fault,
            ErrorEvent::InvalidTss(..) => EventErrorLevel::Fault,
            ErrorEvent::PageFault(..) => EventErrorLevel::Fault,
            ErrorEvent::GeneralProtectionFault(..) => EventErrorLevel::Fault,
            ErrorEvent::DoubleFault(..) => EventErrorLevel::Abort
        }
    }
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