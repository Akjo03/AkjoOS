use alloc::sync::Arc;
use spin::Mutex;
use crate::api::display::{Color, Colors, DisplayApi, Fonts, Position, TextAlignment, TextBaseline, TextLineHeight};

#[allow(dead_code)]
pub enum DisplayDriverType {
    Unknown,
    Dummy(DummyDisplayDriver)
}

trait DisplayDriver {
    fn activate(&mut self, display: Arc<Mutex<dyn DisplayApi + Send>>);
    fn deactivate(&mut self);
}

pub trait CommonDisplayDriver {
    fn new() -> Self;
    fn draw_all(&mut self);

    fn clear(&mut self, color: Color);
}

pub struct DisplayDriverManager {
    pub current_driver: DisplayDriverType
} #[allow(dead_code)] impl DisplayDriverManager {
    pub fn new() -> Self { Self {
        current_driver: DisplayDriverType::Unknown
    } }

    pub fn set_driver(
        &mut self, driver: DisplayDriverType,
        display: Arc<Mutex<dyn DisplayApi + Send>>
    ) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(driver) => {
                driver.deactivate();
            },
            _ => {}
        }
        self.current_driver = driver;
        match &mut self.current_driver {
            DisplayDriverType::Dummy(driver) => {
                driver.activate(display);
            },
            _ => {}
        }
    }

    pub fn clear(&mut self, color: Color) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(driver) => {
                driver.clear(color);
            },
            _ => {}
        }
    }

    pub fn draw_all(&mut self) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(driver) => {
                driver.draw_all();
            },
            _ => {}
        }
    }

    pub fn get_driver(&self) -> &DisplayDriverType {
        &self.current_driver
    }
}

pub struct DummyDisplayDriver {
    display: Option<Arc<Mutex<dyn DisplayApi + Send>>>
} impl DummyDisplayDriver {
    pub fn draw_panic(&mut self, message: &str) {
        if let Some(display) = self.display.as_mut() {
            let mut display = display.try_lock()
                .unwrap_or_else(|| panic!("Failed to lock display for panic message drawing!") );
            display.clear(Colors::Blue.into());
            display.draw_text(
                "Kernel Panic -- please reboot your machine! See message below:", Position::new(0, 0),
                Colors::White.into(), None,
                Fonts::default().into(), false, false,
                TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
            );
            display.draw_text(
                message, Position::new(0, 18),
                Colors::White.into(), None,
                Fonts::Font9x18.into(), false, false,
                TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
            );
            display.swap();
        } else { panic!("No display to draw panic message to!"); }
    }
} impl CommonDisplayDriver for DummyDisplayDriver {
    fn new() -> Self { Self {
        display: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = self.display.as_mut() {
            display.try_lock()
                .unwrap_or_else(|| panic!("Failed to lock display for drawing!") )
                .swap();
        }
    }

    fn clear(&mut self, color: Color) {
        if let Some(display) = self.display.as_mut() {
            let mut display = display.try_lock()
                .unwrap_or_else(|| panic!("Failed to lock display for clearing!") );
            display.clear(color);
            display.swap();
        }
    }
} impl DisplayDriver for DummyDisplayDriver {
    fn activate(&mut self, display: Arc<Mutex<dyn DisplayApi + Send>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}