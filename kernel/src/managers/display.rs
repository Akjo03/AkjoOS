use alloc::fmt;
use alloc::rc::Rc;
use core::cell::RefCell;

use bootloader_api::info::FrameBufferInfo;

use crate::api::display::{Colors, DisplayApi, Fonts};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverManager, DisplayDriverType, DummyDisplayDriver};
use crate::drivers::display::text::{TextDisplayDriver, TextDisplayDriverArgs};
use crate::systems::display::{BufferedDisplay, SimpleDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayMode {
    Unknown,
    Dummy,
    Text(Fonts)
} impl<'a> DisplayMode {
    fn get_driver(self, _info: FrameBufferInfo) -> DisplayDriverType<'a> {
        match self {
            DisplayMode::Unknown => DisplayDriverType::Unknown,
            DisplayMode::Dummy => DisplayDriverType::Dummy(
                DummyDisplayDriver::new()
            ), DisplayMode::Text(font) => DisplayDriverType::Text(
                TextDisplayDriver::new(),
                TextDisplayDriverArgs::new(
                    Rc::new(RefCell::new(font))
                )
            )
        }
    }
} impl fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayMode::Unknown => write!(f, "Unknown"),
            DisplayMode::Dummy => write!(f, "Dummy"),
            DisplayMode::Text(..) => write!(f, "Text")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayType {
    Unknown,
    Simple,
    Buffered
} impl<'a> DisplayType {
    pub fn new(&self, frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Rc<RefCell<dyn DisplayApi + 'a>> {
        match self {
            DisplayType::Unknown => panic!("Unknown display type!"),
            DisplayType::Simple => Rc::new(RefCell::new(
                SimpleDisplay::new(frame_buffer, frame_buffer_info)
            )),
            DisplayType::Buffered => Rc::new(RefCell::new(
                BufferedDisplay::new(frame_buffer, frame_buffer_info)
            ))
        }
    }
} impl fmt::Display for DisplayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayType::Unknown => write!(f, "Unknown"),
            DisplayType::Simple => write!(f, "Simple"),
            DisplayType::Buffered => write!(f, "Buffered")
        }
    }
}

pub struct DisplayManager<'a> {
    display: Rc<RefCell<dyn DisplayApi + 'a>>,
    display_type: DisplayType,
    driver_manager: DisplayDriverManager<'a>
} #[allow(dead_code)] impl<'a> DisplayManager<'a> {
    /// Creates a new display manager. Be careful as multiple display managers will overwrite each other.
    pub fn new(display_type: DisplayType, frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        let display = display_type.new(frame_buffer, frame_buffer_info);
        let driver_manager = DisplayDriverManager::new();

        Self { display, display_type, driver_manager }
    }

    /// Sets the display mode. This will in turn also set the driver for the display.
    pub fn set_mode(&mut self, display_mode: DisplayMode) {
        let driver = display_mode.get_driver(self.display.borrow().get_info());

        match driver {
            DisplayDriverType::Text(..) => {
                let display_type = &self.display_type;
                if display_type != &DisplayType::Buffered {
                    panic!("Text mode is only supported with buffered display!");
                }
            }, _ => {}
        }

        self.driver_manager.set_driver(driver, self.display.clone());
    }

    /// Returns the current driver type, which can be used to get the actual driver.
    pub fn get_driver(&mut self) -> &mut DisplayDriverType<'a> {
        &mut self.driver_manager.current_driver
    }

    /// Returns the current display type.
    pub fn get_display_type(&self) -> DisplayType {
        self.display_type
    }

    /// Returns the current display mode.
    /// Corresponds directly to the current driver type.
    pub fn get_display_mode(&self) -> DisplayMode {
        match &self.driver_manager.current_driver {
            DisplayDriverType::Unknown => DisplayMode::Unknown,
            DisplayDriverType::Dummy(..) => DisplayMode::Dummy,
            DisplayDriverType::Text(..) => DisplayMode::Text(Fonts::default())
        }
    }

    /// Clears the screen.
    pub fn clear_screen(&mut self) {
        self.driver_manager.clear(Colors::Black.into())
    }

    /// Draws all the changes to the screen using the current driver.
    pub fn draw_all(&mut self) {
        self.driver_manager.draw_all()
    }
}