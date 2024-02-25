#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use core::panic::PanicInfo;
use bootloader_api::{BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping;
use x86_64::VirtAddr;
use crate::internal::pic::{PicInterrupts, PicMask};

mod internal;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 1024 * 1024;
    config
};
bootloader_api::entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Initialize serial logger
    internal::serial::init()
        .unwrap_or_else(|err| panic!("Failed to initialize serial logger: {:#?}", err));
    log::info!("Serial logger initialized. Booting AkjoOS...");

    // Initialize memory mapper
    let physical_memory_offset = VirtAddr::new(*boot_info.physical_memory_offset.as_ref()
        .unwrap_or_else(|| panic!("Physical memory offset not found!")));
    let mut mapper = unsafe { internal::memory::init(physical_memory_offset) };
    let usable_region_count = &internal::memory::get_usable_regions(&boot_info.memory_regions, 0).count();
    log::info!(
        "Memory mapper initialized at physical memory offset {:#X}.",
        physical_memory_offset
    );
    log::info!(
        "Detected {} of usable memory regions / frames at 4KiB in size.",
        &usable_region_count
    );

    // Initialize simple heap allocator
    let mut simple_heap_allocator = unsafe {
        internal::heap::SimpleHeapFrameAllocator::new(&boot_info.memory_regions, 0)
    };
    let next = internal::heap::init_initial_heap(&mut mapper, &mut simple_heap_allocator)
        .unwrap_or_else(|err| panic!("Failed to initialize initial heap: {:#?}", err));
    log::info!(
        "Initial heap initialized with {} bytes. Next frame at {}/{}.",
        internal::heap::INITIAL_HEAP_SIZE, next, &usable_region_count
    );

    // Initialize main heap allocator
    let mut frame_allocator = unsafe {
        internal::heap::HeapFrameAllocator::new(&boot_info.memory_regions, next)
    };
    let next = internal::heap::init_main_heap(&mut mapper, &mut frame_allocator)
        .unwrap_or_else(|err| panic!("Failed to initialize main heap: {:#?}", err));
    log::info!(
        "Main heap initialized with {} bytes. Next frame at {}/{}.",
        internal::heap::MAIN_HEAP_SIZE, next, &usable_region_count
    );

    // Switch to main heap
    internal::heap::init_allocator();
    log::info!("Global allocator switched to main heap.");

    // Load GDT table
    internal::gdt::load();
    log::info!("Global descriptor table loaded.");

    // Load ACPI tables and platform information
    let acpi = internal::acpi::load(boot_info.rsdp_addr.into_option(), physical_memory_offset);
    log::info!("ACPI tables loaded.");

    // Load platform info
    let platform_info = acpi.platform_info()
        .unwrap_or_else(|err| panic!("Platform info not found: {:#?}", err));
    let processor_info = platform_info.processor_info()
        .unwrap_or_else(|| panic!("Processor info not found!"));
    log::info!(
        "Platform info loaded with system type '{:?}' and {} processors.",
        platform_info.platform_type(), processor_info.application_processors.iter().count() + 1
    );

    // Initialize PIC8259
    let mut pic_mask = PicMask::new();
    pic_mask.enable(PicInterrupts::Timer);
    internal::pic::init(pic_mask);
    log::info!("Programmable interrupt controller initialized.");

    // Load IDT table
    internal::idt::load();
    log::info!("Interrupt descriptor table loaded and interrupts enabled.");

    // TODO: Do kernel stuff here
    log::info!("Kernel booted successfully.");

    // Disable interrupts
    internal::idt::disable_interrupts();
    log::info!("Interrupts disabled.");

    // Initiate shutdown
    acpi.shutdown().unwrap_or_else(|err| panic!("Failed to initiate shutdown: {:#?}", err));
    log::info!("Shutdown initiated.");

    // Halt CPU
    loop { x86_64::instructions::hlt(); }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let payload_message = if let Some(message) = panic_info.message() {
        message.as_str().unwrap_or("Unknown panic message.")
    } else if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
        payload
    } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
        payload.as_str()
    } else {
        "Unknown panic payload."
    };

    abort(payload_message)
}

fn abort(message: &str) -> ! {
    log::error!("Kernel panicked with message '{}'", message);
    loop { x86_64::instructions::hlt(); }
}