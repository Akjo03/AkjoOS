use core::fmt;
use core::fmt::{Arguments, Write};
use log::{Log, Metadata, Record, SetLoggerError};
use spin::RwLock;
use uart_16550::SerialPort;

static LOGGER: RwLock<Option<SerialPortLogger>> = RwLock::new(None);

struct LoggerWrapper;

#[allow(dead_code)]
pub enum SerialLoggingLevel {
    Debug,
    Info,
    Warning,
    Error,
    Panic
} impl SerialLoggingLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Panic => "PANIC"
        }
    }
}

pub struct SerialPortLogger {
    port: RwLock<SerialPort>
} #[allow(dead_code)] impl SerialPortLogger {
    pub fn init() -> Self {
        let mut port = unsafe { SerialPort::new(0x3F8) };
        port.init();
        Self { port: RwLock::new(port) }
    }

    pub fn log_args(&mut self, args: &Arguments, level: SerialLoggingLevel, file: &str, line: u32) {
        self.port.write().write_fmt(
            format_args!("\n[{}#{} | {}]: {}", file, line, level.as_str(), args)
        ).unwrap();
    }
} impl Write for SerialPortLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.port.write().write_str(s)
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.port.write().write_char(c)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> fmt::Result {
        self.port.write().write_fmt(args)
    }
} impl Log for LoggerWrapper {
    fn enabled(&self, _metadata: &Metadata) -> bool { true }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            log::Level::Trace => SerialLoggingLevel::Debug,
            log::Level::Debug => SerialLoggingLevel::Debug,
            log::Level::Info => SerialLoggingLevel::Info,
            log::Level::Warn => SerialLoggingLevel::Warning,
            log::Level::Error => SerialLoggingLevel::Error
        };

        if let Some(logger) = LOGGER.write().as_mut() {
            logger.log_args(record.args(), level, record.file().unwrap_or("_"), record.line().unwrap_or(0));
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    let mut logger = LOGGER.write();
    if logger.is_none() {
        *logger = Some(SerialPortLogger::init());
    }
    drop(logger);

    log::set_logger(&LoggerWrapper)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
}