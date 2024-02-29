use alloc::sync::Arc;
use spin::Mutex;
use spin::rwlock::RwLock;
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
} impl DisplayMode {
    fn get_driver(self) -> DisplayDriverType {
        match self {
            DisplayMode::Unknown => DisplayDriverType::Unknown,
            DisplayMode::Dummy => DisplayDriverType::Dummy(
                DummyDisplayDriver::new()
            ), DisplayMode::Text(font) => DisplayDriverType::Text(
                TextDisplayDriver::new(),
                TextDisplayDriverArgs::new(
                    Arc::new(RwLock::new(font))
                )
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayType {
    Unknown,
    Simple,
    Buffered
} impl DisplayType {
    pub fn new(&self) -> Arc<Mutex<dyn DisplayApi + Send>> {
        match self {
            DisplayType::Unknown => panic!("Unknown display type!"),
            DisplayType::Simple => Arc::new(Mutex::new(
                SimpleDisplay::new()
            )), DisplayType::Buffered => Arc::new(Mutex::new(
                BufferedDisplay::new()
            ))
        }
    }
}

pub struct DisplayManager {
    display: Arc<Mutex<dyn DisplayApi + Send>>,
    display_type: DisplayType,
    driver_manager: DisplayDriverManager
} #[allow(dead_code)] impl DisplayManager {
    /// Creates a new display manager. Be careful as multiple display managers will overwrite each other.
    pub fn new(display_type: DisplayType) -> Self {
        let display = display_type.new();
        let driver_manager = DisplayDriverManager::new();

        Self { display, display_type, driver_manager }
    }

    /// Sets the display mode. This will in turn also set the driver for the display.
    pub fn set_mode(&mut self, mode: DisplayMode) {
        let driver = mode.get_driver();

        match driver {
            DisplayDriverType::Text(..) => {
                if self.display_type != DisplayType::Buffered {
                    panic!("Text mode can only be used with a buffered display!");
                }
            }, _ => {}
        }

        self.driver_manager.set_driver(driver, self.display.clone());
    }

    /// Returns the current driver type, which can be used to get the actual driver.
    pub fn get_driver(&mut self) -> &mut DisplayDriverType {
        &mut self.driver_manager.current_driver
    }

    /// Returns the current display type.
    pub fn get_display_type(&self) -> DisplayType {
        self.display_type
    }

    /// Clears the screen.
    pub fn clear_screen(&mut self) {
        self.driver_manager.clear(Colors::Black.into());
    }

    /// Draws all the changes to the screen using the current driver.
    pub fn draw_all(&mut self) {
        self.driver_manager.draw_all();
    }
}