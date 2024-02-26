use pic8259::ChainedPics;
use spin::{Mutex, Once};
use x86_64::instructions::port::Port;
use bit_field::BitField;

static DATA_PORT: u16 = 0x40;
static COMMAND_PORT: u16 = 0x43;
static OPERATING_MODE: u8 = 0b0011_0100; // 16-bit binary, rate generator, lo/hi byte, channel 0
pub static TIMER_HZ: u64 = 1000; // 1000Hz (min 19Hz, max 1193180Hz) - 1ms interval
pub static TIMER_DIVISOR: u64 = 1193180 / TIMER_HZ;

static PIC1_OFFSET: u8 = 0x20;
static PIC2_OFFSET: u8 = 0x28;

static PICS: Once<Mutex<ChainedPics>> = Once::new();

#[allow(dead_code)]
pub enum PicInterrupts {
    Timer, Keyboard, PassThrough,
    RTC, ACPI, PCI1, PCI2, Mouse, FPU, PrimaryATA, SecondaryATA,
    COM2, COM1, LPT2, Floppy, LPT1
} impl PicInterrupts {
    pub fn into_values(self) -> (u8, u8) {
        match self {
            PicInterrupts::Timer => (0, PIC1_OFFSET),
            PicInterrupts::Keyboard => (1, PIC1_OFFSET + 1),
            PicInterrupts::PassThrough => (2, PIC1_OFFSET + 2),
            PicInterrupts::RTC => (0, PIC2_OFFSET),
            PicInterrupts::ACPI => (1, PIC2_OFFSET + 1),
            PicInterrupts::PCI1 => (2, PIC2_OFFSET + 2),
            PicInterrupts::PCI2 => (3, PIC2_OFFSET + 3),
            PicInterrupts::Mouse => (4, PIC2_OFFSET + 4),
            PicInterrupts::FPU => (5, PIC2_OFFSET + 5),
            PicInterrupts::PrimaryATA => (6, PIC2_OFFSET + 6),
            PicInterrupts::SecondaryATA => (7, PIC2_OFFSET + 7),
            PicInterrupts::COM2 => (3, PIC1_OFFSET + 3),
            PicInterrupts::COM1 => (4, PIC1_OFFSET + 4),
            PicInterrupts::LPT2 => (5, PIC1_OFFSET + 5),
            PicInterrupts::Floppy => (6, PIC1_OFFSET + 6),
            PicInterrupts::LPT1 => (7, PIC1_OFFSET + 7)
        }
    }
}

pub struct PicMask {
    pic1: u8,
    pic2: u8
} impl PicMask {
    pub fn new() -> Self {
        Self { pic1: 0xFF, pic2: 0xFF }
    }

    pub fn enable(&mut self, interrupt: PicInterrupts) {
        let (mask, offset) = interrupt.into_values();
        if offset < PIC2_OFFSET {
            self.pic1.set_bit(mask as usize, false);
        } else {
            self.pic2.set_bit(mask as usize, false);
        }
    }

    pub fn apply(&self) {
        unsafe {
            PICS.get().unwrap().lock().write_masks(self.pic1, self.pic2);
        }
    }
}

pub fn init(mask: PicMask) {
    PICS.call_once(|| unsafe {
        Mutex::new(ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET))
    });
    mask.apply();
    unsafe {
        let mut pics = PICS.get().unwrap_or_else(|| panic!("PIC not loaded!")).lock();

        let mut data_port: Port<u8> = Port::new(DATA_PORT);
        let mut command_port: Port<u8> = Port::new(COMMAND_PORT);

        let low_byte = (TIMER_DIVISOR & 0xFF) as u8;
        let high_byte = ((TIMER_DIVISOR >> 8) & 0xFF) as u8;

        command_port.write(OPERATING_MODE);
        data_port.write(low_byte);
        data_port.write(high_byte);

        pics.initialize();
    }
}

pub fn end_of_interrupt(interrupt: PicInterrupts) {
    unsafe { PICS.get().unwrap_or_else(|| panic!("PIC not loaded!")).lock().notify_end_of_interrupt(interrupt.into_values().1) }
}