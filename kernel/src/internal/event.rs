use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

#[derive(Debug, Clone)]
pub enum Event {
    Error(ErrorEvent),
}

#[derive(Debug, Clone)]
pub enum ErrorEvent {
    PageFault(String),
    GeneralProtectionFault(String),
    InvalidOpcode(String),
    InvalidTss(String),
    DoubleFault(String),
    Breakpoint(String),
} #[allow(dead_code)] impl ErrorEvent {
    pub fn message(&self) -> &String {
        match self {
            Self::PageFault(message) => message,
            Self::GeneralProtectionFault(message) => message,
            Self::InvalidOpcode(message) => message,
            Self::InvalidTss(message) => message,
            Self::DoubleFault(message) => message,
            Self::Breakpoint(message) => message,
        }
    }

    pub fn level(&self) -> ErrorLevel {
        match self {
            Self::PageFault(_) => ErrorLevel::Fault,
            Self::GeneralProtectionFault(_) => ErrorLevel::Fault,
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

pub trait EventHandler {
    fn handle(&mut self, event: Event);
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
            .for_each(|handler| handler.borrow_mut().handle(event.clone()));
    }
}