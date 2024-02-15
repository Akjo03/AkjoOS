use alloc::borrow::{Cow, ToOwned};
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use embedded_graphics::mono_font::MonoFont;
use crate::api::display::{Color, Colors, DisplayApi, Fonts, Position, Region, Size, TextAlignment, TextBaseline, TextLineHeight};
use crate::drivers::display::{CommonDisplayDriver, DisplayDriver};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TextColor {
    Black = 0, Gray = 8,
    Maroon = 1, Red = 9,
    Green = 2, Lime = 10,
    Olive = 3, Yellow = 11,
    Navy = 4, Blue = 12,
    Purple = 5, Fuchsia = 13,
    Teal = 6, Aqua = 14,
    Silver = 7, White = 15
} impl TextColor {
    #[inline]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(TextColor::Black), 8 => Some(TextColor::Gray),
            1 => Some(TextColor::Maroon), 9 => Some(TextColor::Red),
            2 => Some(TextColor::Green), 10 => Some(TextColor::Lime),
            3 => Some(TextColor::Olive), 11 => Some(TextColor::Yellow),
            4 => Some(TextColor::Navy), 12 => Some(TextColor::Blue),
            5 => Some(TextColor::Purple), 13 => Some(TextColor::Fuchsia),
            6 => Some(TextColor::Teal), 14 => Some(TextColor::Aqua),
            7 => Some(TextColor::Silver), 15 => Some(TextColor::White),
            _ => None
        }
    }
} impl Into<Color> for TextColor {
    fn into(self) -> Color {
        match self {
            TextColor::Black => Colors::Black.into(),
            TextColor::Maroon => Colors::Maroon.into(),
            TextColor::Green => Colors::Green.into(),
            TextColor::Olive => Colors::Olive.into(),
            TextColor::Navy => Colors::Navy.into(),
            TextColor::Purple => Colors::Purple.into(),
            TextColor::Teal => Colors::Teal.into(),
            TextColor::Silver => Colors::Silver.into(),
            TextColor::Gray => Colors::Gray.into(),
            TextColor::Red => Colors::Red.into(),
            TextColor::Lime => Colors::Lime.into(),
            TextColor::Yellow => Colors::Yellow.into(),
            TextColor::Blue => Colors::Blue.into(),
            TextColor::Fuchsia => Colors::Fuchsia.into(),
            TextColor::Aqua => Colors::Aqua.into(),
            TextColor::White => Colors::White.into()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8); impl ColorCode {
    #[inline]
    pub fn new(foreground: TextColor, background: TextColor) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }

    #[inline]
    pub fn foreground(&self) -> TextColor {
        TextColor::from_u8(self.0 & 0xF).unwrap()
    }

    #[inline]
    pub fn background(&self) -> TextColor {
        TextColor::from_u8((self.0 >> 4) & 0xF).unwrap()
    }

    #[inline]
    pub fn invert(&self) -> Self {
        Self((self.0 >> 4) | (self.0 << 4))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct CharacterAttributes(u8); impl CharacterAttributes {
    #[inline]
    pub fn new(underline: bool, strikethrough: bool) -> Self {
        let mut value = 0;
        if underline { value |= 1; }
        if strikethrough { value |= 2; }
        Self(value)
    }

    #[inline]
    pub fn underline(&self) -> bool {
        self.0 & 1 != 0
    }

    #[inline]
    pub fn strikethrough(&self) -> bool {
        self.0 & 2 != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ScreenChar(u32); impl ScreenChar {
    #[inline]
    pub fn new(character: char, color: ColorCode, attributes: CharacterAttributes) -> Self {
        Self((character as u32) | ((color.0 as u32) << 8) | ((attributes.0 as u32) << 16))
    }

    #[inline]
    pub fn character(&self) -> char {
        (self.0 & 0xFF) as u8 as char
    }

    #[inline]
    pub fn color(&self) -> ColorCode {
        ColorCode((self.0 >> 8) as u8)
    }

    #[inline]
    pub fn attributes(&self) -> CharacterAttributes {
        CharacterAttributes((self.0 >> 16) as u8)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegment {
    pub text: Cow<'static, str>,
    pub text_position: Position,
    pub text_color: TextColor,
    pub background_color: TextColor,
    pub underline: bool,
    pub strikethrough: bool
} impl TextSegment {
    #[inline]
    pub fn new(
        text: impl Into<Cow<'static, str>>, text_position: Position,
        text_color: TextColor, background_color: TextColor,
        underline: bool, strikethrough: bool
    ) -> Self { Self {
        text: text.into(), text_position,
        text_color, background_color,
        underline, strikethrough
    } }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up, Down
}

pub const BUFFER_WIDTH: usize = 80;
pub const BUFFER_HEIGHT: usize = 25;

pub struct TextDisplayDriverArgs {
    font: Rc<RefCell<Fonts>>,
} #[allow(dead_code)] impl TextDisplayDriverArgs {
    pub fn new(font: Rc<RefCell<Fonts>>) -> Self {
        Self { font }
    }
}

pub struct TextDisplayDriver<'a> {
    display: Option<Rc<RefCell<dyn DisplayApi + 'a>>>,
    font: Option<Fonts>,
    text_buffer: [ScreenChar; BUFFER_WIDTH * BUFFER_HEIGHT],
    text_cursor: Position,
    dirty_buffer: [bool; BUFFER_WIDTH * BUFFER_HEIGHT],
    text_color: TextColor,
    background_color: TextColor,
    underline: bool,
    strikethrough: bool,
    blink: bool
} #[allow(dead_code)] impl TextDisplayDriver<'_> {
    /// Initializes the text display driver. Should only get called once by the display driver manager.
    pub fn init(&mut self, args: &mut TextDisplayDriverArgs) {
        self.font = Some(args.font.borrow().to_owned());
    }


    /// Writes a character to the text buffer.
    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            '\r' => self.move_cursor(Position::new(0, self.text_cursor.y)),
            '\t' => self.move_cursor(Position::new(self.text_cursor.x + 4, self.text_cursor.y)),
            _ => {
                self.write(ScreenChar::new(
                    character,
                    ColorCode::new(self.text_color, self.background_color),
                    CharacterAttributes::new(self.underline, self.strikethrough)
                ))
            }
        }
    }

    /// Writes a string to the text buffer.
    pub fn write_string(&mut self, text: &str) {
        for character in text.chars() {
            self.write_char(character);
        }
    }

    /// Writes a string to the text buffer and moves the cursor to the next line.
    pub fn write_line(&mut self, text: &str) {
        self.write_string(text);
        self.new_line();
    }

    /// Moves the cursor to the next line.
    pub fn new_line(&mut self) {
        self.move_cursor(Position::new(0, self.text_cursor.y + 1));
    }


    /// Sets the text color for incoming text.
    #[inline]
    pub fn set_text_color(&mut self, color: TextColor) {
        self.text_color = color;
    }

    /// Sets the background color for incoming text.
    #[inline]
    pub fn set_background_color(&mut self, color: TextColor) {
        self.background_color = color;
    }

    /// Sets the underline attribute for incoming text.
    #[inline]
    pub fn set_underline(&mut self, underline: bool) {
        self.underline = underline;
    }

    /// Sets the strikethrough attribute for incoming text.
    #[inline]
    pub fn set_strikethrough(&mut self, strikethrough: bool) {
        self.strikethrough = strikethrough;
    }


    /// Moves the cursor to a specific position.
    #[inline]
    pub fn move_cursor(&mut self, position: Position) {
        self.text_cursor = position;
    }

    /// Retrieves the current cursor position.
    #[inline]
    pub fn get_cursor_position(&self) -> Position {
        self.text_cursor
    }


    /// Clears a specific cell in the text buffer.
    pub fn clear_cell(&mut self, row: usize, col: usize) {
        let index = row * BUFFER_WIDTH + col;
        self.text_buffer[index] = ScreenChar::new(
            ' ',
            ColorCode::new(self.background_color, self.background_color),
            CharacterAttributes::new(false, false),
        );
        self.dirty_buffer[index] = true;
    }

    /// Clears the entire text buffer.
    pub fn clear_buffer(&mut self) {
        self.text_buffer.fill(ScreenChar::new(
            ' ',
            ColorCode::new(TextColor::Black, TextColor::Black),
            CharacterAttributes::new(false, false)
        ));
        self.move_cursor(Position::new(0, 0));
    }


    /// Fills the entire text buffer with a specific character.
    pub fn fill(&mut self, character: char) {
        let screen_char = ScreenChar::new(
            character,
            ColorCode::new(self.text_color, self.background_color),
            CharacterAttributes::new(self.underline, self.strikethrough)
        );

        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let index = row * BUFFER_WIDTH + col;
                self.text_buffer[index] = screen_char;
                self.dirty_buffer[index] = true;
            }
        }
    }

    /// Fills a specific region in the text buffer with a specific character.
    pub fn fill_region(&mut self, region: Region, character: char) {
        let screen_char = ScreenChar::new(
            character,
            ColorCode::new(self.text_color, self.background_color),
            CharacterAttributes::new(self.underline, self.strikethrough)
        );

        for row in region.position.y..(region.position.y + region.size.height) {
            for col in region.position.x..(region.position.x + region.size.width) {
                let index = row * BUFFER_WIDTH + col;
                self.text_buffer[index] = screen_char;
                self.dirty_buffer[index] = true;
            }
        }
    }


    /// Scrolls the text buffer by a specific amount of lines in a specific direction.
    pub fn scroll(&mut self, lines: usize, direction: ScrollDirection) {
        if lines == 0 { return; }

        if lines >= BUFFER_HEIGHT {
            self.clear_buffer();
            return;
        }

        match direction {
            ScrollDirection::Up => {
                for row in 0..(BUFFER_HEIGHT - lines) {
                    for col in 0..BUFFER_WIDTH {
                        let from_index = (row + lines) * BUFFER_WIDTH + col;
                        let to_index = row * BUFFER_WIDTH + col;
                        self.text_buffer[to_index] = self.text_buffer[from_index];
                        self.dirty_buffer[to_index] = true;
                    }
                }
                for row in (BUFFER_HEIGHT - lines)..BUFFER_HEIGHT {
                    for col in 0..BUFFER_WIDTH {
                        self.clear_cell(row, col);
                    }
                }

                self.move_cursor(Position::new(self.text_cursor.x, self.text_cursor.y - lines));
            }, ScrollDirection::Down => {
                for row in (lines..BUFFER_HEIGHT).rev() {
                    for col in 0..BUFFER_WIDTH {
                        let from_index = (row - lines) * BUFFER_WIDTH + col;
                        let to_index = row * BUFFER_WIDTH + col;
                        self.text_buffer[to_index] = self.text_buffer[from_index];
                        self.dirty_buffer[to_index] = true;
                    }
                }
                for row in 0..lines {
                    for col in 0..BUFFER_WIDTH {
                        self.clear_cell(row, col);
                    }
                }
            }
        }
    }

    /// Toggles the blink attribute for the text cursor.
    pub fn blink(&mut self) {
        self.blink = !self.blink;
    }


    /// Initializes the whole text buffer to be redrawn on the next draw call.
    pub fn init_redraw(&mut self) {
        self.dirty_buffer.fill(true);
    }

    /// Validates a specific position in the text buffer.
    ///
    /// Returns a tuple with two booleans, the first one indicating if the x position is valid
    /// and the second one indicating if the y position is valid.
    #[inline]
    pub fn validate_position(&mut self, position: Position) -> (bool, bool) {
        (position.x < BUFFER_WIDTH, position.y < BUFFER_HEIGHT)
    }

    /// Validates a specific region in the text buffer.
    #[inline]
    pub fn validate_region(&mut self, region: Region) -> bool {
        let (x_valid, y_valid) = self.validate_position(region.position);

        let end_x = region.position.x + region.size.width;
        let end_y = region.position.y + region.size.height;

        let x_valid_end = end_x < BUFFER_WIDTH;
        let y_valid_end = end_y < BUFFER_HEIGHT;

        x_valid && y_valid && x_valid_end && y_valid_end
    }


    #[inline]
    fn write(&mut self, character: ScreenChar) {
        let mut new_position = self.text_cursor;

        loop {
            match self.validate_position(new_position) {
                (true, true) => {
                    self.write_at(character, new_position);
                    new_position.x += 1;
                    break;
                }, (false, true) => {
                    new_position.x = 0;
                    new_position.y += 1;
                }, _ => {
                    self.scroll(1, ScrollDirection::Up);
                    new_position = self.text_cursor;
                }
            }
        }

        self.move_cursor(new_position);
    }

    #[inline]
    fn write_at(&mut self, character: ScreenChar, position: Position) {
        let index = position.y * BUFFER_WIDTH + position.x;
        self.text_buffer[index] = character;
        self.dirty_buffer[index] = true;
    }


    fn get_text_segments(&mut self) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let dirty_regions = self.get_dirty_regions();

        for region in dirty_regions.iter() {
            let start_x = region.position.x;
            let start_y = region.position.y;
            let end_x = start_x + region.size.width;
            let end_y = start_y + region.size.height;

            let mut current_text = String::new();
            let mut current_position = Position::new(start_x, start_y);
            let mut current_text_color = self.text_color;
            let mut current_background_color = self.background_color;
            let mut current_underline = false;
            let mut current_strikethrough = false;
            let mut last_x = start_x;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    if x == 0 && last_x != 0 && !current_text.is_empty() {
                        segments.push(TextSegment::new(
                            current_text.clone(), current_position,
                            current_text_color, current_background_color,
                            current_underline, current_strikethrough
                        ));
                        current_text.clear();
                    }

                    let index = y * BUFFER_WIDTH + x;
                    let screen_char = self.text_buffer[index];
                    let char_color = screen_char.color();
                    let char_attributes = screen_char.attributes();

                    if current_text.is_empty() {
                        current_text_color = char_color.foreground();
                        current_background_color = char_color.background();
                        current_underline = char_attributes.underline();
                        current_strikethrough = char_attributes.strikethrough();
                        current_text.push(screen_char.character());
                        current_position = Position::new(x, y);
                    } else if (current_text_color != char_color.foreground() || current_background_color != char_color.background() ||
                        current_underline != char_attributes.underline() || current_strikethrough != char_attributes.strikethrough()) &&
                        (current_text_color == TextColor::Black && current_background_color == TextColor::Black) {
                        segments.push(TextSegment::new(
                            current_text.clone(), current_position,
                            current_text_color, current_background_color,
                            current_underline, current_strikethrough
                        ));

                        current_text = screen_char.character().to_string();
                        current_position = Position::new(x, y);
                        current_text_color = char_color.foreground();
                        current_background_color = char_color.background();
                        current_underline = char_attributes.underline();
                        current_strikethrough = char_attributes.strikethrough();
                    } else {
                        current_text.push(screen_char.character());
                    }

                    last_x = x;
                }
                if !current_text.is_empty() {
                    segments.push(TextSegment::new(
                        current_text.clone(), current_position,
                        current_text_color, current_background_color,
                        current_underline, current_strikethrough
                    ));
                    current_text.clear();
                }
                last_x = 0;
            }
        }

        segments
    }

    fn get_dirty_regions(&mut self) -> Vec<Region> {
        let mut regions = Vec::new();
        let mut visited = [false; BUFFER_WIDTH * BUFFER_HEIGHT];

        for y in 0..BUFFER_HEIGHT {
            for x in 0..BUFFER_WIDTH {
                let index = y * BUFFER_WIDTH + x;
                if self.dirty_buffer[index] && !visited[index] {
                    let mut bounds = (x, x, y, y);
                    self.dfs(x, y, &mut visited, &mut bounds);

                    let region = Region::new(
                        Position::new(bounds.0, bounds.2),
                        Size::new(bounds.1 - bounds.0 + 1, bounds.3 - bounds.2 + 1),
                    );
                    regions.push(region);
                }
            }
        }

        regions
    }

    fn dfs(&mut self, x: usize, y: usize, visited: &mut [bool; BUFFER_WIDTH * BUFFER_HEIGHT], bounds: &mut (usize, usize, usize, usize)) {
        let index = y * BUFFER_WIDTH + x;
        if x >= BUFFER_WIDTH || y >= BUFFER_HEIGHT || visited[index] || !self.dirty_buffer[index] {
            return;
        }

        visited[index] = true;
        bounds.0 = bounds.0.min(x);
        bounds.1 = bounds.1.max(x);
        bounds.2 = bounds.2.min(y);
        bounds.3 = bounds.3.max(y);

        if x > 0 { self.dfs(x - 1, y, visited, bounds); }
        if x < BUFFER_WIDTH - 1 { self.dfs(x + 1, y, visited, bounds); }
        if y > 0 { self.dfs(x, y - 1, visited, bounds); }
        if y < BUFFER_HEIGHT - 1 { self.dfs(x, y + 1, visited, bounds); }
    }


    fn map_position(&mut self, text_position: Position) -> Position {
        if let Some(font) = self.font.as_ref() {
            let font: MonoFont = (*font).into();

            let screen_x = text_position.x * font.character_size.width as usize;
            let screen_y = text_position.y * font.character_size.height as usize;
            return Position::new(screen_x, screen_y);
        }

        Position::new(0, 0)
    }
} impl<'a> CommonDisplayDriver<'a> for TextDisplayDriver<'a> {
    fn new() -> Self { Self {
        display: None,
        font: None,
        text_buffer: [ScreenChar::new(
            ' ',
            ColorCode::new(TextColor::Black, TextColor::Black),
            CharacterAttributes::new(false, false)
        ); BUFFER_WIDTH * BUFFER_HEIGHT],
        text_cursor: Position::new(0, 0),
        dirty_buffer: [false; BUFFER_WIDTH * BUFFER_HEIGHT],
        text_color: TextColor::White,
        background_color: TextColor::Black,
        underline: false,
        strikethrough: false,
        blink: false
    } }

    fn draw_all(&mut self) {
        let segments = self.get_text_segments();

        let pre_calculated_positions: Vec<(Cow<'static, str>, Position, Color, Color, bool, bool)> = segments.iter().map(|segment| {
            let screen_position = self.map_position(segment.text_position);
            let text_color: Color = segment.text_color.into();
            let background_color: Color = segment.background_color.into();
            (segment.text.clone(), screen_position, text_color, background_color, segment.underline, segment.strikethrough)
        }).collect();

        let cursor_position = self.map_position(self.text_cursor);

        let display_opt = self.display.as_mut();
        let font_opt = self.font.as_ref();

        if let (Some(display), Some(font)) = (display_opt, font_opt) {
            let mut display = display.borrow_mut();
            let font: MonoFont = (*font).into();

            for (text, screen_position, text_color, background_color, underline, strikethrough) in pre_calculated_positions {
                display.draw_text(
                    &text, screen_position,
                    text_color, Some(background_color),
                    font, underline, strikethrough,
                    TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                );
            }

            if self.blink {
                let color_code = ColorCode::new(self.text_color, self.background_color);

                display.draw_char(
                    ' ', cursor_position,
                    color_code.invert().foreground().into(), Some(color_code.invert().background().into()),
                    font, false, false,
                    TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                );
            } else {
                display.draw_char(
                    ' ', cursor_position,
                    self.text_color.into(), Some(self.background_color.into()),
                    font, false, false,
                    TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
                );
            }

            display.swap();
        }
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
} impl<'a> DisplayDriver<'a> for TextDisplayDriver<'a> {
    fn activate(&mut self, display: Rc<RefCell<dyn DisplayApi + 'a>>) {
        self.display = Some(display);
    }

    fn deactivate(&mut self) {
        self.display = None;
    }
}