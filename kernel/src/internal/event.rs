use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use crate::internal::serial::SerialLoggingLevel;

#[derive(Debug, Clone, Copy)]
pub enum Event {
    TimerInterrupt,
    Error(ErrorEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorEvent {
    PageFault,
    GeneralProtectionFault,
    InvalidOpcode,
    InvalidTss,
    DoubleFault,
    Breakpoint,
}

fn mask_index(event: Event) -> u8 {
    match event {
        Event::TimerInterrupt => 0,
        Event::Error(ErrorEvent::PageFault) => 1,
        Event::Error(ErrorEvent::GeneralProtectionFault) => 2,
        Event::Error(ErrorEvent::InvalidOpcode) => 3,
        Event::Error(ErrorEvent::InvalidTss) => 4,
        Event::Error(ErrorEvent::DoubleFault) => 5,
        Event::Error(ErrorEvent::Breakpoint) => 6,
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EventMask {
    mask: u64,
} #[allow(dead_code)] impl EventMask {
    #[inline]
    pub fn new() -> EventMask { Self {
        mask: 0
    } }

    #[inline]
    pub fn all() -> EventMask { Self {
        mask: u64::MAX
    } }

    #[inline]
    pub fn set(&mut self, event: Event) {
        self.mask |= 1 << mask_index(event);
    }

    #[inline]
    pub fn read(&self, event: Event) -> bool {
        self.mask & (1 << mask_index(event)) != 0
    }
}

pub trait EventHandler {
    fn handle(&mut self, event: Event);
    fn mask(&self) -> EventMask;
}

pub struct EventDispatcher {
    handlers: Vec<Rc<RefCell<dyn EventHandler>>>,
} impl EventDispatcher {
    pub fn new() -> EventDispatcher { Self {
        handlers: Vec::new()
    } }

    pub fn register(&mut self, handler: Rc<RefCell<dyn EventHandler>>) {
        self.handlers.push(handler);
    }

    pub fn dispatch(&self, event: Event) {
        if let Some(serial_logger) = crate::get_serial_logger() {
            serial_logger.log(format_args!(
                "Dispatching event {:?}.", event
            ), SerialLoggingLevel::Info);
        }

        self.handlers.iter()
            .filter(|handler| handler.borrow().mask().read(event))
            .for_each(|handler| handler.borrow_mut().handle(event));
    }
}