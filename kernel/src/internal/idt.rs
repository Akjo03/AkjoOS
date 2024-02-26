use alloc::format;
use spin::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::internal::event::{ErrorEvent, Event};
use crate::internal::pic::PicInterrupts;

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn load() {
    IDT.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        // Hardware Interrupt Handlers
        idt[PicInterrupts::Timer.into_values().1 as usize].set_handler_fn(timer_interrupt_handler);
        idt[PicInterrupts::RTC.into_values().1 as usize].set_handler_fn(rtc_interrupt_handler);

        // Exception Handlers
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(super::gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler)
                .set_stack_index(super::gdt::PAGE_FAULT_IST_INDEX);
            idt.general_protection_fault.set_handler_fn(general_protection_fault_handler)
                .set_stack_index(super::gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);
        }

        idt
    });

    IDT.get().unwrap_or_else(|| panic!("Interrupt descriptor table not found!")).load();

    x86_64::instructions::interrupts::enable();
}

pub fn without_interrupts<F, R>(func: F) -> R
    where F: FnOnce() -> R {
    x86_64::instructions::interrupts::without_interrupts(func)
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

// Hardware Interrupt Handlers

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    crate::internal::event::EventDispatcher::global().push(Event::Timer);
    crate::internal::pic::end_of_interrupt(PicInterrupts::Timer);
}

extern "x86-interrupt" fn rtc_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    let date_time = crate::internal::cmos::Cmos::global()
        .unwrap_or_else(|| panic!("CMOS not found!"))
        .lock().rtc();
    crate::internal::event::EventDispatcher::global().push(Event::Rtc(date_time));
    crate::internal::pic::end_of_interrupt(PicInterrupts::RTC);
}

// Exception Handlers

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame
) { crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::Breakpoint(
    format!("{:#?}", stack_frame)
))) }

extern "x86-interrupt" fn invalid_opcode_handler(
    stack_frame: InterruptStackFrame
) { crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::InvalidOpcode(
    format!("{:#?}", stack_frame)
))) }


extern "x86-interrupt" fn invalid_tss_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) { crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::InvalidTss(
    format!("{:#?}", stack_frame), error_code
))) }

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode
) { crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::PageFault(
    format!("{:#?}", stack_frame), error_code.bits()
))) }

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) { crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::GeneralProtectionFault(
    format!("{:#?}", stack_frame), error_code
))) }

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, error_code: u64
) -> ! {
    crate::internal::event::EventDispatcher::global().push(Event::error(ErrorEvent::DoubleFault(
        format!("{:#?}", stack_frame), error_code
    )));
    loop { x86_64::instructions::hlt(); }
}