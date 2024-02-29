#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::sync::Arc;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use bootloader_api::{BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping;
use spin::Mutex;
use x86_64::VirtAddr;
use crate::api::event::{ErrorEvent, Event, EventHandler};
use crate::drivers::display::DisplayDriverType;
use crate::internal::pic::{PicInterrupts, PicMask};
use crate::managers::display::{DisplayManager, DisplayMode, DisplayType};
use crate::managers::time::TimeManager;

mod internal;
mod kernel;

mod api;
mod systems;
mod drivers;
mod managers;

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

    // Load FADT table
    let fadt = acpi.fadt()
        .unwrap_or_else(|err| panic!("FADT table not found: {:#?}", err));
    log::info!("FADT table loaded.");

    // Initialize PIC8259
    let mut pic_mask = PicMask::new();
    pic_mask.enable(PicInterrupts::Timer);
    pic_mask.enable(PicInterrupts::PassThrough);
    pic_mask.enable(PicInterrupts::RTC);
    internal::pic::init(pic_mask);
    log::info!("Programmable interrupt controller initialized.");

    // Initialize CMOS and enable interrupts
    internal::cmos::init(fadt.century);
    internal::cmos::Cmos::global()
        .unwrap_or_else(|| panic!("CMOS not found!"))
        .lock().enable_interrupts();
    log::info!("CMOS initialized and CMOS interrupts enabled.");

    // Load IDT table
    internal::idt::load();
    log::info!("Interrupt descriptor table loaded and interrupts enabled.");

    // Initialize frame buffer
    if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
        let info = frame_buffer.info().clone();
        let buffer = frame_buffer.buffer_mut();

        internal::framebuffer::init(info, buffer);
        log::info!(
            "Frame buffer initialized with resolution {}x{} and {}bpp.",
            info.width, info.height, info.bytes_per_pixel * 8
        )
    }

    // Initialize time manager
    let time_manager = TimeManager::new();
    log::info!("Time manager initialized.");

    // Initialize display manager
    let mut display_manager = DisplayManager::new(DisplayType::Buffered);
    display_manager.set_mode(DisplayMode::Dummy);
    display_manager.clear_screen();
    log::info!("Display manager initialized.");

    // Initialize kernel
    let kernel = Arc::new(Mutex::new(Kernel::new(
        time_manager,
        display_manager
    )));
    kernel.lock().init();
    api::event::EventDispatcher::global().register(kernel.clone());
    log::info!("Kernel initialized and registered as event handler.");

    // Main kernel loop
    log::info!("Kernel booted successfully. Entering main loop...");
    while kernel.lock().running.load(Ordering::SeqCst) {
        api::event::EventDispatcher::global().dispatch();
    }

    log::info!("Kernel needs to stop running. Shutting down...");

    // Disable interrupts
    internal::idt::disable_interrupts();
    log::info!("Interrupts disabled.");

    // Halt kernel
    kernel.lock().halt();
    log::info!("Kernel halted.");

    // Initiate shutdown
    acpi.shutdown().unwrap_or_else(|err| panic!("Failed to initiate shutdown: {:#?}", err));
    log::info!("Shutdown initiated.");

    // Halt CPU
    loop { x86_64::instructions::hlt(); }
}

#[allow(dead_code)]
pub struct Kernel {
    time_manager: TimeManager,
    display_manager: DisplayManager,
    pub tick: AtomicU64,
    pub running: AtomicBool
} impl Kernel {
    pub fn new(
        time_manager: TimeManager,
        display_manager: DisplayManager
    ) -> Self { Self {
        time_manager,
        display_manager,
        tick: AtomicU64::new(0),
        running: AtomicBool::new(true)
    } }
} impl EventHandler for Kernel {
    fn handle(&mut self, event: Event) {
        match event {
            Event::Timer => {
                self.tick.fetch_add(1, Ordering::SeqCst);
                self.tick();
            },
            Event::Error(event) => self.on_error(event),
            _ => {}
        }
    }
}

pub trait KernelRuntime {
    fn init(&mut self);
    fn tick(&mut self);
    fn on_error(&mut self, event: ErrorEvent);
    fn halt(&mut self);
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

    internal::framebuffer::is_initialized().then(|| {
        let mut display_manager = DisplayManager::new(DisplayType::Simple);
        display_manager.set_mode(DisplayMode::Dummy);
        display_manager.clear_screen();

        abort(payload_message, Some(&mut display_manager));
    });

    abort(payload_message, None);
}

fn abort(message: &str, display_manager: Option<&mut DisplayManager>) -> ! {
    log::error!("Kernel panicked with message '{}'", message);

    if let Some(display_manager) = display_manager {
        match display_manager.get_driver() {
            DisplayDriverType::Dummy(driver) => {
                driver.draw_panic(message);
            }, _ => {}
        }
        display_manager.draw_all();
    }

    loop { x86_64::instructions::hlt(); }
}