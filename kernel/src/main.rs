#![feature(exclusive_range_pattern)]
#![feature(panic_info_message)]
#![feature(const_mut_refs)]
#![feature(abi_x86_interrupt)]
#![feature(asm_const)]
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
use crate::internal::event::{EventDispatcher, EventHandler};
use crate::internal::memory::SimpleBootInfoFrameAllocator;
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
    init_serial_logger();

    if let Some(serial_logger) = get_serial_logger() {
        serial_logger.log(format_args!(
            "Serial port initialized. Booting kernel of AkjoOS..."
        ), SerialLoggingLevel::Info);

        internal::gdt::init();
        serial_logger.log(format_args!(
            "Global descriptor table initialized."
        ), SerialLoggingLevel::Info);

        internal::idt::init();
        serial_logger.log(format_args!(
            "Interrupt descriptor table initialized."
        ), SerialLoggingLevel::Info);

        if let Some(frame_buffer) = boot_info.framebuffer.as_mut() {
            let info = frame_buffer.info().clone();
            let buffer = frame_buffer.buffer_mut();
            initialize_frame_buffer(buffer, info);

            serial_logger.log(format_args!(
                "Frame buffer initialized with resolution {}x{} at {}bpp.",
                info.width, info.height, info.bytes_per_pixel * 8
            ), SerialLoggingLevel::Info);
        } else { panic!("Frame buffer not found!") }

        let physical_memory_offset = boot_info.physical_memory_offset.as_ref()
            .expect("Physical memory offset not found!");
        let physical_memory_offset = VirtAddr::new(*physical_memory_offset);
        let mut mapper = unsafe { internal::memory::init(physical_memory_offset) };
        let usable_region_count = &internal::memory::get_usable_regions(&boot_info.memory_regions, 0).count();

        serial_logger.log(format_args!(
            "Memory mapper initialized at physical memory offset {:?}.",
            physical_memory_offset
        ), SerialLoggingLevel::Info);
        serial_logger.log(format_args!(
            "Detected {} of usable memory regions / frames at 4KiB in size.",
            &usable_region_count
        ), SerialLoggingLevel::Info);

        let mut simple_frame_allocator = unsafe {
            SimpleBootInfoFrameAllocator::new(&boot_info.memory_regions)
        };
        let next = internal::memory::init_initial_heap(&mut mapper, &mut simple_frame_allocator)
            .expect("Failed to initialize initial heap!");
        serial_logger.log(format_args!(
            "Initial heap initialized with {} bytes. Next frame at {}/{}.",
            internal::memory::INITIAL_HEAP_SIZE, next, &usable_region_count
        ), SerialLoggingLevel::Info);

        let mut frame_allocator = unsafe {
            internal::memory::BootInfoFrameAllocator::new(&boot_info.memory_regions, next)
        };
        let next = internal::memory::init_main_heap(&mut mapper, &mut frame_allocator)
            .expect("Failed to initialize main heap!");
        internal::memory::init_allocator();

        serial_logger.log(format_args!(
            "Main kernel heap initialized with {} bytes and global allocator switched to it. Stopped at frame {}/{}.",
            internal::memory::MAIN_HEAP_SIZE, next, &usable_region_count
        ), SerialLoggingLevel::Info);

        init_event_dispatcher();
        serial_logger.log(format_args!(
            "Event dispatcher initialized."
        ), SerialLoggingLevel::Info);

        if let Some(frame_buffer) = get_framebuffer() {
            if let Some(frame_buffer_info) = get_framebuffer_info() {
                let mut display_manager = DisplayManager::new(
                    DisplayType::Buffered, frame_buffer, frame_buffer_info
                );
                display_manager.set_mode(DisplayMode::Dummy);
                display_manager.clear_screen();

                serial_logger.log(format_args!(
                    "Display manager initialized using display mode {} and type {}.",
                    display_manager.get_display_mode(), display_manager.get_display_type()
                ), SerialLoggingLevel::Info);

                let kernel = Rc::new(RefCell::new(Kernel::new(
                    get_serial_logger().unwrap(),
                    display_manager
                )));

                kernel.borrow_mut().init();

                if let Some(event_dispatcher) = get_event_dispatcher() {
                    event_dispatcher.register(Rc::clone(&kernel) as Rc<RefCell<dyn EventHandler>>);

                    while kernel.borrow().running.load(Ordering::Relaxed) {
                        x86_64::instructions::hlt();
                    }
                }

                serial_logger.log(format_args!(
                    "Kernel needs to stop running. Shutting down..."
                ), SerialLoggingLevel::Info);

                internal::idt::disable();
                serial_logger.log(format_args!(
                    "Interrupt descriptor table disabled."
                ), SerialLoggingLevel::Info);

                kernel.borrow_mut().halt();
                serial_logger.log(format_args!(
                    "Kernel halted."
                ), SerialLoggingLevel::Info);

                loop { x86_64::instructions::hlt(); }
            } else { panic!("Frame buffer info not found!") }
        } else { panic!("Frame buffer not found!") }
    } else { panic!("Serial port not found!") }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(serial_port) = get_serial_logger() {
        serial_port.log(format_args!("{:?}", panic_info.payload()), SerialLoggingLevel::Panic);
    }

    if let Some(frame_buffer) = get_framebuffer() {
        if let Some(frame_buffer_info) = get_framebuffer_info() {
            let mut display_manager = DisplayManager::new(DisplayType::Simple, frame_buffer, frame_buffer_info);
            display_manager.set_mode(DisplayMode::Dummy);

            match display_manager.get_driver() {
                DisplayDriverType::Dummy(driver) => {
                    let mut message_found = false;

                    if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
                        driver.draw_panic(payload);
                        message_found = true;
                    } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
                        driver.draw_panic(payload.as_str());
                        message_found = true;
                    } else if let Some(message) = panic_info.message() {
                        if let Some(message_str) = message.as_str() {
                            driver.draw_panic(message_str);
                            message_found = true;
                        }
                    }

                    if !message_found {
                        driver.draw_panic("No message provided!");
                    }
                }, _ => {}
            }
        }
    }

    loop { x86_64::instructions::hlt(); }
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