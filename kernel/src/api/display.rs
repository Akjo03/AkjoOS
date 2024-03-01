use bootloader_api::info::FrameBufferInfo;
use embedded_graphics::{
    geometry::Point,
    mono_font::{
        ascii::*, MonoFont,
    },
    pixelcolor::Rgb888, text::{
        Alignment, Baseline, LineHeight
    },
    primitives::Rectangle,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
} #[allow(dead_code)] impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }
} #[allow(dead_code)] impl Into<Point> for Position {
    fn into(self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
} #[allow(dead_code)] impl Size {
    pub fn new(width: usize, height: usize) -> Size {
        Size { width, height }
    }
} #[allow(dead_code)] impl Into<embedded_graphics::geometry::Size> for Size {
    fn into(self) -> embedded_graphics::geometry::Size {
        embedded_graphics::geometry::Size::new(self.width as u32, self.height as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub position: Position,
    pub size: Size,
} #[allow(dead_code)] impl Region {
    pub fn new(position: Position, size: Size) -> Region {
        Region { position, size }
    }
} #[allow(dead_code)] impl Into<Rectangle> for Region {
    fn into(self) -> Rectangle {
        Rectangle::new(self.position.into(), self.size.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
} #[allow(dead_code)] impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
        Color { red, green, blue }
    }
} #[allow(dead_code)] impl Into<Rgb888> for Color {
    fn into(self) -> Rgb888 {
        Rgb888::new(self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Colors {
    Black, Silver, Gray, White,
    Maroon, Red, Purple, Fuchsia,
    Green, Lime, Olive, Yellow,
    Navy, Blue, Teal, Aqua,
} #[allow(dead_code)] impl Into<Color> for Colors {
    fn into(self) -> Color {
        match self {
            Colors::Black => Color::new(0, 0, 0),
            Colors::Silver => Color::new(192, 192, 192),
            Colors::Gray => Color::new(128, 128, 128),
            Colors::White => Color::new(255, 255, 255),
            Colors::Maroon => Color::new(128, 0, 0),
            Colors::Red => Color::new(255, 0, 0),
            Colors::Purple => Color::new(128, 0, 128),
            Colors::Fuchsia => Color::new(255, 0, 255),
            Colors::Green => Color::new(0, 128, 0),
            Colors::Lime => Color::new(0, 255, 0),
            Colors::Olive => Color::new(128, 128, 0),
            Colors::Yellow => Color::new(255, 255, 0),
            Colors::Navy => Color::new(0, 0, 128),
            Colors::Blue => Color::new(0, 0, 255),
            Colors::Teal => Color::new(0, 128, 128),
            Colors::Aqua => Color::new(0, 255, 255),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Fonts {
    Font6x9, Font6x10, Font6x12,
    Font6x13, Font6x13B, Font6x13I,
    Font7x13, Font7x13B, Font7x13I,
    Font7x14, Font7x14B,
    Font8x13, Font8x13B, Font8x13I,
    Font9x15, Font9x15B,
    Font9x18, Font9x18B,
    Font10x20,
} #[allow(dead_code)] impl Fonts {
    pub fn get_size(self) -> Size { match self {
        Fonts::Font6x9 => Size::new(6, 9),
        Fonts::Font6x10 => Size::new(6, 10),
        Fonts::Font6x12 => Size::new(6, 12),
        Fonts::Font6x13 => Size::new(6, 13),
        Fonts::Font6x13B => Size::new(6, 13),
        Fonts::Font6x13I => Size::new(6, 13),
        Fonts::Font7x13 => Size::new(7, 13),
        Fonts::Font7x13B => Size::new(7, 13),
        Fonts::Font7x13I => Size::new(7, 13),
        Fonts::Font7x14 => Size::new(7, 14),
        Fonts::Font7x14B => Size::new(7, 14),
        Fonts::Font8x13 => Size::new(8, 13),
        Fonts::Font8x13B => Size::new(8, 13),
        Fonts::Font8x13I => Size::new(8, 13),
        Fonts::Font9x15 => Size::new(9, 15),
        Fonts::Font9x15B => Size::new(9, 15),
        Fonts::Font9x18 => Size::new(9, 18),
        Fonts::Font9x18B => Size::new(9, 18),
        Fonts::Font10x20 => Size::new(10, 20),
    }}
} #[allow(dead_code)] impl Into<MonoFont<'_>> for Fonts {
    fn into(self) -> MonoFont<'static> { match self {
        Fonts::Font6x9 => FONT_6X9,
        Fonts::Font6x10 => FONT_6X10,
        Fonts::Font6x12 => FONT_6X12,
        Fonts::Font6x13 => FONT_6X13,
        Fonts::Font6x13B => FONT_6X13_BOLD,
        Fonts::Font6x13I => FONT_6X13_ITALIC,
        Fonts::Font7x13 => FONT_7X13,
        Fonts::Font7x13B => FONT_7X13_BOLD,
        Fonts::Font7x13I => FONT_7X13_ITALIC,
        Fonts::Font7x14 => FONT_7X14,
        Fonts::Font7x14B => FONT_7X14_BOLD,
        Fonts::Font8x13 => FONT_8X13,
        Fonts::Font8x13B => FONT_8X13_BOLD,
        Fonts::Font8x13I => FONT_8X13_ITALIC,
        Fonts::Font9x15 => FONT_9X15,
        Fonts::Font9x15B => FONT_9X15_BOLD,
        Fonts::Font9x18 => FONT_9X18,
        Fonts::Font9x18B => FONT_9X18_BOLD,
        Fonts::Font10x20 => FONT_10X20,
    } }
} impl Default for Fonts {
    fn default() -> Self { Fonts::Font10x20 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TextBaseline {
    Top, Middle, Bottom,
} #[allow(dead_code)] impl Into<Baseline> for TextBaseline {
    fn into(self) -> Baseline { match self {
        TextBaseline::Top => Baseline::Top,
        TextBaseline::Middle => Baseline::Middle,
        TextBaseline::Bottom => Baseline::Bottom,
    } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TextAlignment {
    Left, Center, Right,
} #[allow(dead_code)] impl Into<Alignment> for TextAlignment {
    fn into(self) -> Alignment { match self {
        TextAlignment::Left => Alignment::Left,
        TextAlignment::Center => Alignment::Center,
        TextAlignment::Right => Alignment::Right,
    } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TextLineHeight {
    Half, Full, Double,
    Pixels(u32),
    Percent(u32),
} #[allow(dead_code)] impl Into<LineHeight> for TextLineHeight {
    fn into(self) -> LineHeight { match self {
        TextLineHeight::Half => LineHeight::Percent(50),
        TextLineHeight::Full => LineHeight::Percent(100),
        TextLineHeight::Double => LineHeight::Percent(200),
        TextLineHeight::Pixels(pixels) => LineHeight::Pixels(pixels),
        TextLineHeight::Percent(percent) => LineHeight::Percent(percent),
    } }
}

pub trait DisplayApi {
    /// Draws the given buffer to the display without modification.
    fn draw(&mut self, buffer: &[u8]);
    /// Draws a single character to the display at the given position with the given style.
    fn draw_char(
        &mut self, character: char, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    );
    /// Draws a string to the display at the given position with the given style.
    /// Does not wrap or scroll the text.
    fn draw_text(
        &mut self, text: &str, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    );
    /// Overwrites the entire display with the given color.
    fn clear(&mut self, color: Color);
    /// Swaps the front and back buffers, displaying the changes made since the last swap.
    /// Only applicable to displays with multiple buffers.
    fn swap(&mut self);
    /// Returns the information about the frame buffer.
    fn get_info(&self) -> FrameBufferInfo;
}