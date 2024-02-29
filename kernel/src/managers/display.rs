use alloc::sync::Arc;
use spin::Mutex;
use crate::api::display::{Colors, DisplayApi};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriverManager, DisplayDriverType, DummyDisplayDriver};
use crate::systems::display::{BufferedDisplay, SimpleDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DisplayMode {
    Unknown,
    Dummy
} impl DisplayMode {
    fn get_driver(self) -> DisplayDriverType {
        match self {
            DisplayMode::Dummy => DisplayDriverType::Dummy(
                DummyDisplayDriver::new()
            ), _ => DisplayDriverType::Unknown
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
    pub fn new(display_type: DisplayType) -> Self {
        let display = display_type.new();
        let driver_manager = DisplayDriverManager::new();

        Self { display, display_type, driver_manager }
    }

    pub fn set_mode(&mut self, mode: DisplayMode) {
        let driver = mode.get_driver();

        self.driver_manager.set_driver(driver, self.display.clone());
    }

    pub fn get_driver(&mut self) -> &mut DisplayDriverType {
        &mut self.driver_manager.current_driver
    }

    pub fn get_display_type(&self) -> DisplayType {
        self.display_type
    }

    pub fn clear_screen(&mut self) {
        self.driver_manager.clear(Colors::Black.into());
    }

    pub fn draw_all(&mut self) {
        self.driver_manager.draw_all();
    }
}