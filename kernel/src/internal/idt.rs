use alloc::format;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::internal::event::{ErrorEvent, Event};

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

        idt
    };
}

pub fn init() {
    IDT.load();
    x86_64::instructions::interrupts::enable();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
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