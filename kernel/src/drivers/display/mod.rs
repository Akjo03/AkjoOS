use alloc::rc::Rc;
use core::cell::RefCell;

use crate::api::display::{Color, Colors, DisplayApi, Fonts, Position, Size, TextAlignment, TextBaseline, TextLineHeight};
use crate::drivers::display::text::{TextDisplayDriver, TextDisplayDriverArgs};

pub mod text;

pub struct DisplayDriverManager<'a> {
    pub current_driver: DisplayDriverType<'a>
} #[allow(dead_code)] impl<'a> DisplayDriverManager<'a> {
    pub fn new() -> Self { Self {
        current_driver: DisplayDriverType::Unknown
    } }

    pub fn set_driver(&mut self, driver: DisplayDriverType<'a>, display: Rc<RefCell<dyn DisplayApi + 'a>>) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.deactivate();
            }, DisplayDriverType::Text(ref mut driver, ..) => {
                driver.deactivate();
            }, _ => {}
        }
        self.current_driver = driver;
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.activate(display);
            }, DisplayDriverType::Text(ref mut driver, args) => {
                driver.init(args);
                driver.activate(display);
            }, _ => {}
        }
    }

    pub fn clear(&mut self, color: Color) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.clear(color);
            }, DisplayDriverType::Text(ref mut driver, ..) => {
                driver.clear(color);
            }, _ => {}
        }
    }

    pub fn draw_all(&mut self) {
        match &mut self.current_driver {
            DisplayDriverType::Dummy(ref mut driver) => {
                driver.draw_all();
            }, DisplayDriverType::Text(ref mut driver, ..) => {
                driver.draw_all();
            }, _ => {}
        }
    }

    pub fn get_driver(&self) -> &DisplayDriverType<'a> {
        &self.current_driver
    }
}

#[allow(dead_code)]
pub enum DisplayDriverType<'a> {
    Unknown,
    Dummy(DummyDisplayDriver<'a>),
    Text(TextDisplayDriver<'a>, TextDisplayDriverArgs)
}

trait DisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<dyn DisplayApi + 'a>>);
    fn deactivate(&mut self);
}

pub trait CommonDisplayDriver<'a> {
    fn new() -> Self;
    /// Draws all changes to the display.
    fn draw_all(&mut self);

    /// Clears the display to a specific color.
    fn clear(&mut self, color: Color);
    /// Returns the size of the display.
    fn get_size(&self) -> Size;
}

pub struct DummyDisplayDriver<'a> {
    display: Option<Rc<RefCell<dyn DisplayApi + 'a>>>,
} #[allow(dead_code)] impl DummyDisplayDriver<'_> {
    pub fn draw_panic(&mut self, message: &str) {
        if let Some(display) = self.display.as_mut() {
            let mut display = display.borrow_mut();
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
} impl<'a> CommonDisplayDriver<'a> for DummyDisplayDriver<'a> {
    fn new() -> Self { Self {
        display: None
    } }

    fn draw_all(&mut self) {
        if let Some(display) = self.display.as_mut() {
            display.borrow_mut().swap();
        } else { panic!("No display to draw to!"); }
    }

    fn clear(&mut self, color: Color) {
        if let Some(display) = self.display.as_mut() {
            let mut display = display.borrow_mut();
            display.clear(color);
            display.swap();
        } else { panic!("No display to clear!"); }
    }

    fn get_size(&self) -> Size {
        if let Some(display) = self.display.as_ref() {
            let info = display.borrow().get_info();
            Size::new(info.width, info.height)
        } else { Size::new(0, 0) }
    }
} impl<'a> DisplayDriver<'a> for DummyDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<dyn DisplayApi + 'a>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}