use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::internal::serial::SerialLoggingLevel;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
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
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Timer interrupt occurred."
    ), SerialLoggingLevel::Info);

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()) };
} }

// Exception Handlers

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Breakpoint exception occurred: {:#?}",
        stack_frame
    ), SerialLoggingLevel::Error)
} }

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Invalid opcode exception occurred: {:#?}",
        stack_frame
    ), SerialLoggingLevel::Error)
} }

extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Invalid TSS exception occurred with error code {:#?}: {:#?}",
        error_code, stack_frame
    ), SerialLoggingLevel::Error)
} }

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Page fault exception occurred with error code {:#?}: {:#?}",
        error_code, stack_frame
    ), SerialLoggingLevel::Error)
} }

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "General protection fault exception occurred with error code {:#?}: {:#?}",
        error_code, stack_frame
    ), SerialLoggingLevel::Error)
} }

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! { if let Some(serial_logger) = crate::get_serial_logger() {
    serial_logger.log(format_args!(
        "Double fault exception occurred: {:#?}",
        stack_frame
    ), SerialLoggingLevel::Error)
} loop {} }