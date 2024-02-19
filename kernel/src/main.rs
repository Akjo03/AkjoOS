#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;
use core::panic::PanicInfo;
use core::sync::atomic::Ordering;
use bootloader_api::{BootInfo, BootloaderConfig};
use bootloader_api::config::Mapping;
use bootloader_api::info::FrameBufferInfo;
use x86_64::VirtAddr;
use crate::drivers::display::DisplayDriverType;
use crate::internal::event::{Event, EventDispatcher, EventHandler};
use crate::internal::memory::SimpleHeapFrameAllocator;
use crate::internal::serial::{SerialLoggingLevel, SerialPortLogger};
use crate::kernel::Kernel;
use crate::managers::display::{DisplayManager, DisplayMode, DisplayType};

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
    init_serial_logger();

    if let Some(serial_logger) = get_serial_logger() {
        serial_logger.log(&format_args!(
            "Serial port initialized. Booting kernel of AkjoOS..."
        ), SerialLoggingLevel::Info);

        // Initialize memory mapper
        let physical_memory_offset = boot_info.physical_memory_offset.as_ref()
            .expect("Physical memory offset not found!");
        let physical_memory_offset = VirtAddr::new(*physical_memory_offset);
        let mut mapper = unsafe { internal::memory::init(physical_memory_offset) };
        let usable_region_count = &internal::memory::get_usable_regions(&boot_info.memory_regions, 0).count();
        serial_logger.log(&format_args!(
            "Memory mapper initialized at physical memory offset {:?}.",
            physical_memory_offset
        ), SerialLoggingLevel::Info);
        serial_logger.log(&format_args!(
            "Detected {} of usable memory regions / frames at 4KiB in size.",
            &usable_region_count
        ), SerialLoggingLevel::Info);

        // Initialize simple heap allocator
        let mut simple_heap_allocator = unsafe {
            SimpleHeapFrameAllocator::new(&boot_info.memory_regions, 0)
        };
        let next = internal::memory::init_initial_heap(&mut mapper, &mut simple_heap_allocator)
            .expect("Failed to initialize initial heap!");
        serial_logger.log(&format_args!(
            "Initial heap initialized with {} bytes. Next frame at {}/{}.",
            internal::memory::INITIAL_HEAP_SIZE, next, &usable_region_count
        ), SerialLoggingLevel::Info);

        // Initialize main heap allocator
        let mut frame_allocator = unsafe {
            internal::memory::HeapFrameAllocator::new(&boot_info.memory_regions, next)
        };
        let next = internal::memory::init_main_heap(&mut mapper, &mut frame_allocator)
            .expect("Failed to initialize main heap!");
        internal::memory::init_allocator();
        serial_logger.log(&format_args!(
            "Main kernel heap initialized with {} bytes and global allocator switched to it. Stopped at frame {}/{}.",
            internal::memory::MAIN_HEAP_SIZE, next, &usable_region_count
        ), SerialLoggingLevel::Info);

        // Load GDT table
        internal::gdt::load();
        serial_logger.log(&format_args!(
            "Global descriptor table loaded."
        ), SerialLoggingLevel::Info);

        // Load ACPI tables
        let acpi_tables = &internal::acpi::load(Option::from(boot_info.rsdp_addr), physical_memory_offset);
        serial_logger.log(&format_args!(
            "ACPI tables loaded."
        ), SerialLoggingLevel::Info);

        // Load MADT table
        let _madt_table = &internal::madt::load(acpi_tables, physical_memory_offset);
        serial_logger.log(&format_args!(
            "MADT table loaded."
        ), SerialLoggingLevel::Info);

        // Load IDT table
        internal::idt::load();
        serial_logger.log(&format_args!(
            "Interrupt descriptor table loaded."
        ), SerialLoggingLevel::Info);

        // Initialize Frame Buffer
        if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
            let info = frame_buffer.info().clone();
            let buffer = frame_buffer.buffer_mut();
            initialize_frame_buffer(buffer, info);

            serial_logger.log(&format_args!(
                "Frame buffer initialized with resolution {}x{} at {}bpp.",
                info.width, info.height, info.bytes_per_pixel * 8
            ), SerialLoggingLevel::Info);
        } else { panic!("Frame buffer not found!") }

        // Initialize event dispatcher
        init_event_dispatcher();
        serial_logger.log(&format_args!(
            "Event dispatcher initialized."
        ), SerialLoggingLevel::Info);

        // Initialize and run kernel
        if let Some(frame_buffer) = get_framebuffer() {
            if let Some(frame_buffer_info) = get_framebuffer_info() {
                // Initialize display manager
                let mut display_manager = DisplayManager::new(
                    DisplayType::Buffered, frame_buffer, frame_buffer_info
                );
                display_manager.set_mode(DisplayMode::Dummy);
                display_manager.clear_screen();
                serial_logger.log(&format_args!(
                    "Display manager initialized using display mode {} and type {}.",
                    display_manager.get_display_mode(), display_manager.get_display_type()
                ), SerialLoggingLevel::Info);

                // Create kernel instance
                let kernel = Rc::new(RefCell::new(Kernel::new(
                    get_serial_logger().unwrap(),
                    display_manager
                )));

                kernel.borrow_mut().init();

                // Main kernel loop and register event handler
                if let Some(event_dispatcher) = get_event_dispatcher() {
                    event_dispatcher.register(Rc::clone(&kernel) as Rc<RefCell<dyn EventHandler>>);

                    while kernel.borrow().running.load(Ordering::Relaxed) {
                        kernel.borrow_mut().tick.fetch_add(1, Ordering::Relaxed);
                        kernel.borrow_mut().tick();
                    }
                }

                serial_logger.log(&format_args!(
                    "Kernel needs to stop running. Shutting down..."
                ), SerialLoggingLevel::Info);

                // Halt kernel
                kernel.borrow_mut().halt();
                serial_logger.log(&format_args!(
                    "Kernel halted."
                ), SerialLoggingLevel::Info);

                // Loop forever
                loop { x86_64::instructions::hlt(); }
            } else { panic!("Frame buffer info not found!") }
        } else { panic!("Frame buffer not found!") }
    } else { panic!("Serial port not found!") }
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

    if let Some(frame_buffer) = get_framebuffer() {
        if let Some(frame_buffer_info) = get_framebuffer_info() {
            let mut display_manager = DisplayManager::new(
                DisplayType::Buffered, frame_buffer, frame_buffer_info
            );
            display_manager.set_mode(DisplayMode::Dummy);
            display_manager.clear_screen();

            abort(payload_message, Some(&mut display_manager));
        }
    }

    abort(payload_message, None);
}

fn abort(message: &str, display_manager: Option<&mut DisplayManager>) -> ! {
    if let Some(serial_port) = get_serial_logger() {
        serial_port.log(&format_args!("{}", message), SerialLoggingLevel::Panic);
    }

    if let Some(display_manager) = display_manager {
        display_manager.set_mode(DisplayMode::Dummy);
        match display_manager.get_driver() {
            DisplayDriverType::Dummy(driver) => {
                driver.draw_panic(message);
            }, _ => {}
        }
        display_manager.draw_all();
    }

    loop { x86_64::instructions::hlt(); }
}

// -------- "Hidden" Event Handler Implementation for Kernel --------

impl EventHandler for Kernel<'_> {
    fn handle(&mut self, event: Event) {
        match event {
            Event::Error(event) => {
                self.on_error(event);
            },
        }
    }
}

// -------- Static Access to Serial Logger --------

static mut SERIAL_LOGGER: Option<SerialPortLogger> = None;

fn init_serial_logger() { unsafe {
    SERIAL_LOGGER = Some(SerialPortLogger::init());
} }

fn get_serial_logger() -> Option<&'static mut SerialPortLogger> {
    unsafe { SERIAL_LOGGER.as_mut() }
}

// -------- Static Access to Framebuffer --------

static mut FRAMEBUFFER: Option<&'static mut [u8]> = None;
static mut FRAMEBUFFER_INFO: Option<FrameBufferInfo> = None;

fn initialize_frame_buffer(frame_buffer: &'static mut [u8], frame_buffer_info: FrameBufferInfo) { unsafe {
    FRAMEBUFFER = Some(frame_buffer);
    FRAMEBUFFER_INFO = Some(frame_buffer_info);
} }

fn get_framebuffer() -> Option<&'static mut [u8]> {
    unsafe { FRAMEBUFFER.as_mut().map(|fb| &mut **fb) }
}

fn get_framebuffer_info() -> Option<FrameBufferInfo> {
    unsafe { FRAMEBUFFER_INFO }
}

// -------- Static Access to Event Dispatcher --------

static mut EVENT_DISPATCHER: Option<EventDispatcher> = None;

fn init_event_dispatcher() { unsafe {
    EVENT_DISPATCHER = Some(EventDispatcher::new());
} }

fn get_event_dispatcher() -> Option<&'static mut EventDispatcher> {
    unsafe { EVENT_DISPATCHER.as_mut() }
}