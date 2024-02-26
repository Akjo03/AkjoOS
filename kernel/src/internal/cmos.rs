use core::fmt::Display;
use core::hint::spin_loop;
use bit_field::BitField;
use spin::{Mutex, Once};
use x86_64::instructions::port::Port;

static CENTURY: u16 = 2000;

static CMOS_PORT_1: u16 = 0x70;
static CMOS_PORT_2: u16 = 0x71;

static CMOS: Once<Mutex<Cmos>> = Once::new();

#[repr(u8)]
#[derive(Debug, Clone)]
enum CmosRegister {
    Seconds = 0x00,
    Minutes = 0x02,
    Hours = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    StatusA = 0x0A,
    StatusB = 0x0B,
    StatusC = 0x0C
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16
} impl Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02}/{:02}/{:04} {:02}:{:02}:{:02}", self.month, self.day, self.year, self.hours, self.minutes, self.seconds)
    }
}

pub struct Cmos {
    port_1: Port<u8>,
    port_2: Port<u8>,
    century_register: u8
} impl Cmos {
    pub(crate) fn global() -> &'static Mutex<Self> {
        CMOS.get().expect("CMOS not initialized!")
    }

    fn new(century_register: u8) -> Self { Self {
        port_1: Port::new(CMOS_PORT_1),
        port_2: Port::new(CMOS_PORT_2),
        century_register
    } }

    fn read_date_time(&mut self) -> DateTime {
        while self.read_register(CmosRegister::StatusC as u8) & 0x80 != 0 {}
        let seconds = self.read_register(CmosRegister::Seconds as u8);
        let minutes = self.read_register(CmosRegister::Minutes as u8);
        let hours = self.read_register(CmosRegister::Hours as u8);
        let day = self.read_register(CmosRegister::Day as u8);
        let month = self.read_register(CmosRegister::Month as u8);
        let year = self.read_register(CmosRegister::Year as u8) as u16;
        DateTime { seconds, minutes, hours, day, month, year }
    }

    pub fn rtc(&mut self) -> DateTime {
        self.disable_nmi();
        let mut rtc;
        loop {
            self.wait_for_update();
            rtc = self.read_date_time();
            self.wait_for_update();
            if rtc == self.read_date_time() { break; }
        }

        let status_b = self.read_register(CmosRegister::StatusB as u8);

        if status_b & 0x04 == 0 {
            rtc.seconds = (rtc.seconds & 0x0F) + ((rtc.seconds / 16) * 10);
            rtc.minutes = (rtc.minutes & 0x0F) + ((rtc.minutes / 16) * 10);
            rtc.hours = ((rtc.hours & 0x0F) + (((rtc.hours & 0x70) / 16) * 10))
                | (rtc.hours & 0x80);
            rtc.day = (rtc.day & 0x0F) + ((rtc.day / 16) * 10);
            rtc.month = (rtc.month & 0x0F) + ((rtc.month / 16) * 10);
            rtc.year = (rtc.year & 0x0F) + ((rtc.year / 16) * 10);
        }

        if (status_b & 0x02 == 0) && (rtc.hours & 0x80 == 0) {
            rtc.hours = ((rtc.hours & 0x07F) + 12) % 24;
        }

        rtc.year += match self.century_register {
            0 => 1900,
            0x32 => 2000,
            _ => CENTURY
        };

        self.enable_nmi();
        rtc
    }

    pub fn enable_interrupts(&mut self) {
        crate::internal::idt::without_interrupts(|| {
            self.disable_nmi();
            let prev = self.read_register(CmosRegister::StatusB as u8);
            self.write_register(CmosRegister::StatusB, 0x8B);
            self.write_register(CmosRegister::StatusB, prev | 0x40);
            self.enable_nmi();
            self.notify_end_of_interrupt();
        })
    }

    pub fn notify_end_of_interrupt(&mut self) {
        self.read_register(CmosRegister::StatusC as u8);
    }

    fn wait_for_update(&mut self) {
        while self.updating() { spin_loop() }
    }

    fn updating(&mut self) -> bool {
        self.read_register(CmosRegister::StatusA as u8).get_bit(7)
    }

    fn enable_nmi(&mut self) {
        let status_b = self.read_register(CmosRegister::StatusB as u8) & 0x7F;
        self.write_register(CmosRegister::StatusB, status_b)
    }

    fn disable_nmi(&mut self) {
        let status_b = self.read_register(CmosRegister::StatusB as u8) | 0x80;
        self.write_register(CmosRegister::StatusB, status_b)
    }

    fn read_register(&mut self, register: u8) -> u8 { unsafe {
        self.port_1.write(register);
        self.port_2.read()
    } }

    fn write_register(&mut self, register: CmosRegister, value: u8) { unsafe {
        self.port_1.write(register as u8);
        self.port_2.write(value)
    } }
}

pub fn init(century_register: u8) {
    CMOS.call_once(|| Mutex::new(Cmos::new(century_register)));
}