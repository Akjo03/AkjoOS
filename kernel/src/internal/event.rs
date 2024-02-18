use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use crate::internal::serial::SerialLoggingLevel;

#[derive(Debug, Clone)]
pub enum Event {
    TimerInterrupt,
    Error(ErrorEvent),
}

#[derive(Debug, Clone)]
pub enum ErrorEvent {
    PageFault(String),
    GeneralProtectionFault(String, u64),
    InvalidOpcode(String),
    InvalidTss(String),
    DoubleFault(String),
    Breakpoint(String),
} #[allow(dead_code)] impl ErrorEvent {
    pub fn message(&self) -> &String {
        match self {
            Self::PageFault(message) => message,
            Self::GeneralProtectionFault(message, _) => message,
            Self::InvalidOpcode(message) => message,
            Self::InvalidTss(message) => message,
            Self::DoubleFault(message) => message,
            Self::Breakpoint(message) => message,
        }
    }

    pub fn error_code(&self) -> Option<u64> {
        match self {
            Self::PageFault(_) => None,
            Self::GeneralProtectionFault(_, error_code) => Some(*error_code),
            Self::InvalidOpcode(_) => None,
            Self::InvalidTss(_) => None,
            Self::DoubleFault(_) => None,
            Self::Breakpoint(_) => None,
        }
    }

    pub fn level(&self) -> ErrorLevel {
        match self {
            Self::PageFault(_) => ErrorLevel::Fault,
            Self::GeneralProtectionFault(_, _) => ErrorLevel::Fault,
            Self::InvalidOpcode(_) => ErrorLevel::Fault,
            Self::InvalidTss(_) => ErrorLevel::Fault,
            Self::DoubleFault(_) => ErrorLevel::Abort,
            Self::Breakpoint(_) => ErrorLevel::Trap,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ErrorLevel {
    Interrupt,
    Trap,
    Fault,
    Abort,
}

fn mask_index(event: Event) -> u8 {
    match event {
        Event::TimerInterrupt => 0,
        Event::Error(ErrorEvent::PageFault(_)) => 1,
        Event::Error(ErrorEvent::GeneralProtectionFault(_, _)) => 2,
        Event::Error(ErrorEvent::InvalidOpcode(_)) => 3,
        Event::Error(ErrorEvent::InvalidTss(_)) => 4,
        Event::Error(ErrorEvent::DoubleFault(_)) => 5,
        Event::Error(ErrorEvent::Breakpoint(_)) => 6,
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
        self.handlers.iter()
            .filter(|handler| handler.borrow().mask().read(event.clone()))
            .for_each(|handler| handler.borrow_mut().handle(event.clone()));
    }
}