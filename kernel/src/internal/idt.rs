use alloc::format;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::internal::event::{ErrorEvent, Event};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
} impl InterruptIndex {
    #[inline]
    pub fn as_u8(self) -> u8 { self as u8 }

    #[inline]
    pub fn as_usize(self) -> usize { usize::from(self.as_u8()) }
}

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Exception Handlers

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);

        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        // Hardware Interrupt Handlers

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize(); }
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
    unsafe { PICS.lock().disable() };
}

// Hardware Interrupt Handlers

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    if let Some(event_dispatcher) = crate::get_event_dispatcher() {
        let event = Event::TimerInterrupt;

        event_dispatcher.dispatch(event);
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()) };
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Some(event_dispatcher) = crate::get_event_dispatcher() {
        let event = Event::KeyboardInterrupt(scancode);

        event_dispatcher.dispatch(event);
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()) };
}

// Exception Handlers

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::Breakpoint(
        format!("{:#?}", stack_frame)
    ));

    event_dispatcher.dispatch(event);
} }

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame
) { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::InvalidOpcode(
        format!("{:#?}", stack_frame)
    ));

    event_dispatcher.dispatch(event);
} }

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::InvalidTss(
        format!("{:#?}, error code: {:#?}", stack_frame, error_code)
    ));

    event_dispatcher.dispatch(event);
} }

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode
) { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::PageFault(
        format!("{:#?}, error code: {:#?}", stack_frame, error_code)
    ));

    event_dispatcher.dispatch(event);
} }

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::GeneralProtectionFault(
        format!("{:#?}", stack_frame)
    ));

    event_dispatcher.dispatch(event);
} }

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! { if let Some(event_dispatcher) = crate::get_event_dispatcher() {
    let event = Event::Error(ErrorEvent::DoubleFault(
        format!("{:#?}", stack_frame)
    ));

    event_dispatcher.dispatch(event);
} loop { x86_64::instructions::hlt(); } }