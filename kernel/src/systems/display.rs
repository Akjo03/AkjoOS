use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use embedded_graphics::geometry::{Dimensions, Point};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::{Drawable, Pixel};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::{DecorationColor, Text, TextStyle};
use embedded_graphics::text::renderer::CharacterStyle;
use crate::api::display::{Color, DisplayApi, Position, TextAlignment, TextBaseline, TextLineHeight};

trait DisplayContext<'a> {
    fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self;
    fn set_pixel(&mut self, position: Position, color: Color);
    fn swap(&mut self);
}

pub struct SimpleDisplay<'a> {
    context: SimpleDisplayContext<'a>
} impl<'a> SimpleDisplay<'a> {
    pub fn new(
        frame_buffer: &'a mut [u8],
        frame_buffer_info: FrameBufferInfo
    ) -> Self { Self {
        context: SimpleDisplayContext::new(frame_buffer, frame_buffer_info)
    } }
} impl DisplayApi for SimpleDisplay<'_> {
    fn draw(&mut self, buffer: &[u8]) {
        if buffer.len() != self.context.frame_buffer.len() {
            panic!("Frame buffer data does not match the expected size!");
        }

        for (i, byte) in buffer.iter().enumerate() {
            self.context.frame_buffer[i] = *byte;
        }
    }

    fn draw_char(
        &mut self, character: char, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    ) {
        let mut font_style = MonoTextStyle::new(&font, text_color.into());
        font_style.background_color = background_color.map(|color| color.into());

        if underline { font_style.set_underline_color(DecorationColor::TextColor); }
        if strikethrough { font_style.set_strikethrough_color(DecorationColor::TextColor); }

        let mut text_style = TextStyle::default();
        text_style.baseline = baseline.into();
        text_style.alignment = alignment.into();
        text_style.line_height = line_height.into();

        let binding = character.to_string();
        let text = Text::with_text_style(
            &*binding, Point::new(position.x as i32, position.y as i32),
            font_style, text_style
        );

        if let Err(_) = text.draw(&mut self.context) {
            panic!("Failed to draw character!")
        }
    }

    fn draw_text(
        &mut self, text: &str, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    ) {
        let mut font_style = MonoTextStyle::new(&font, text_color.into());
        font_style.background_color = background_color.map(|color| color.into());

        if underline { font_style.set_underline_color(DecorationColor::TextColor); }
        if strikethrough { font_style.set_strikethrough_color(DecorationColor::TextColor); }

        let mut text_style = TextStyle::default();
        text_style.baseline = baseline.into();
        text_style.alignment = alignment.into();
        text_style.line_height = line_height.into();

        let text = Text::with_text_style(
            text, Point::new(position.x as i32, position.y as i32),
            font_style, text_style
        );

        if let Err(_) = text.draw(&mut self.context) {
            panic!("Failed to draw text!")
        }
    }

    fn clear(&mut self, color: Color) {
        for byte_offset in (0..self.context.frame_buffer.len()).step_by(self.context.frame_buffer_info.bytes_per_pixel) {
            set_pixel_in_at(self.context.frame_buffer, self.context.frame_buffer_info, byte_offset, color);
        }
    }

    fn swap(&mut self) { self.context.swap(); }

    fn get_info(&self) -> FrameBufferInfo { self.context.frame_buffer_info }
}

pub struct BufferedDisplay<'a> {
    context: BufferedDisplayContext<'a>
} impl<'a> BufferedDisplay<'a> {
    pub fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        Self { context: BufferedDisplayContext::new(frame_buffer, frame_buffer_info) }
    }
} impl DisplayApi for BufferedDisplay<'_> {
    fn draw(&mut self, buffer: &[u8]) {
        if buffer.len() != self.context.back_buffer.len() {
            panic!("Buffer data does not match the expected size!");
        }

        for (i, byte) in buffer.iter().enumerate() {
            self.context.back_buffer[i] = *byte;
        }
    }

    fn draw_char(
        &mut self, character: char, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    ) {
        let mut font_style = MonoTextStyle::new(&font, text_color.into());
        font_style.background_color = background_color.map(|color| color.into());

        if underline { font_style.set_underline_color(DecorationColor::TextColor); }
        if strikethrough { font_style.set_strikethrough_color(DecorationColor::TextColor); }

        let mut text_style = TextStyle::default();
        text_style.baseline = baseline.into();
        text_style.alignment = alignment.into();
        text_style.line_height = line_height.into();

        let binding = character.to_string();
        let text = Text::with_text_style(
            &*binding, Point::new(position.x as i32, position.y as i32),
            font_style, text_style
        );

        if let Err(_) = text.draw(&mut self.context) {
            panic!("Failed to draw character!")
        }
    }

    fn draw_text(
        &mut self, text: &str, position: Position,
        text_color: Color, background_color: Option<Color>,
        font: MonoFont, underline: bool, strikethrough: bool,
        baseline: TextBaseline, alignment: TextAlignment, line_height: TextLineHeight
    ) {
        let mut font_style = MonoTextStyle::new(&font, text_color.into());
        font_style.background_color = background_color.map(|color| color.into());

        if underline { font_style.set_underline_color(DecorationColor::TextColor); }
        if strikethrough { font_style.set_strikethrough_color(DecorationColor::TextColor); }

        let mut text_style = TextStyle::default();
        text_style.baseline = baseline.into();
        text_style.alignment = alignment.into();
        text_style.line_height = line_height.into();

        let text = Text::with_text_style(
            text, Point::new(position.x as i32, position.y as i32),
            font_style, text_style
        );

        if let Err(_) = text.draw(&mut self.context) {
            panic!("Failed to draw text!")
        }
    }

    fn clear(&mut self, color: Color) {
        for byte_offset in (0..self.context.frame_buffer.len()).step_by(self.context.frame_buffer_info.bytes_per_pixel) {
            set_pixel_in_at(self.context.back_buffer.as_mut_slice(), self.context.frame_buffer_info, byte_offset, color);
        }
    }

    fn swap(&mut self) { self.context.swap(); }

    fn get_info(&self) -> FrameBufferInfo { self.context.frame_buffer_info }
}

struct SimpleDisplayContext<'a> {
    frame_buffer: &'a mut [u8],
    frame_buffer_info: FrameBufferInfo
} impl<'a> DisplayContext<'a> for SimpleDisplayContext<'a> {
    fn new(
        frame_buffer: &'a mut [u8],
        frame_buffer_info: FrameBufferInfo
    ) -> Self { Self {
        frame_buffer,
        frame_buffer_info
    } }

    fn set_pixel(&mut self, position: Position, color: Color) {
        let byte_offset = {
            let line_offset = position.y * self.frame_buffer_info.stride;
            let pixel_offset = line_offset + position.x;
            pixel_offset * self.frame_buffer_info.bytes_per_pixel
        };

        set_pixel_in_at(self.frame_buffer, self.frame_buffer_info, byte_offset, color);
    }

    fn swap(&mut self) {}
} impl DrawTarget for SimpleDisplayContext<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {

        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            self.set_pixel(Position::new(
                point.x as usize,
                point.y as usize
            ), Color::new(
                color.r(),
                color.g(),
                color.b()
            ));
        }

        Ok(())
    }
} impl Dimensions for SimpleDisplayContext<'_> {
    fn bounding_box(&self) -> Rectangle {
        get_bounds(self.frame_buffer_info)
    }
}

struct BufferedDisplayContext<'a> {
    frame_buffer: &'a mut [u8],
    back_buffer: Vec<u8>,
    frame_buffer_info: FrameBufferInfo
} impl<'a> DisplayContext<'a> for BufferedDisplayContext<'a> {
    fn new(frame_buffer: &'a mut [u8], frame_buffer_info: FrameBufferInfo) -> Self {
        let back_buffer = vec![0; frame_buffer.len()];
        Self {
            frame_buffer,
            back_buffer,
            frame_buffer_info
        }
    }

    fn set_pixel(&mut self, position: Position, color: Color) {
        let byte_offset = {
            let line_offset = position.y * self.frame_buffer_info.stride;
            let pixel_offset = line_offset + position.x;
            pixel_offset * self.frame_buffer_info.bytes_per_pixel
        };

        set_pixel_in_at(self.back_buffer.as_mut_slice(), self.frame_buffer_info, byte_offset, color);
    }

    fn swap(&mut self) {
        let frame_buffer_len = self.frame_buffer.len();
        let back_buffer_len = self.back_buffer.len();

        if frame_buffer_len != back_buffer_len {
            panic!("Frame buffer and back buffer sizes do not match!");
        }

        self.frame_buffer.copy_from_slice(&self.back_buffer);
    }
} impl DrawTarget for BufferedDisplayContext<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where I: IntoIterator<Item = Pixel<Self::Color>> {

        for pixel in pixels.into_iter() {
            let Pixel(point, color) = pixel;
            self.set_pixel(Position::new(
                point.x as usize,
                point.y as usize
            ), Color::new(
                color.r(),
                color.g(),
                color.b()
            ));
        }

        Ok(())
    }
} impl Dimensions for BufferedDisplayContext<'_> {
    fn bounding_box(&self) -> Rectangle {
        get_bounds(self.frame_buffer_info)
    }
}

fn get_bounds(frame_buffer_info: FrameBufferInfo) -> Rectangle {
    Rectangle::new(
        Point::new(0, 0),
        embedded_graphics::geometry::Size::new(
            frame_buffer_info.width as u32,
            frame_buffer_info.height as u32
        )
    )
}

fn set_pixel_in_at(frame_buffer: &mut [u8], frame_buffer_info: FrameBufferInfo, index: usize, color: Color) {
    let pixel_buffer = &mut frame_buffer[index..index + frame_buffer_info.bytes_per_pixel];

    match frame_buffer_info.pixel_format {
        PixelFormat::Rgb => {
            pixel_buffer[0] = color.red;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.blue;
        }, PixelFormat::Bgr => {
            pixel_buffer[0] = color.blue;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.red;
        }, PixelFormat::U8 => {
            let gray = color.red / 3 + color.green / 3 + color.blue / 3;
            pixel_buffer[0] = gray;
        }, other => panic!("Unsupported pixel format: {:?}", other)
    }
}