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

trait DisplayContext {
    fn new() -> Self;
    fn set_pixel(&mut self, position: Position, color: Color);
    fn swap(&mut self);
}

pub struct SimpleDisplay {
    context: SimpleDisplayContext
} impl SimpleDisplay {
    pub fn new() -> Self {
        Self { context: SimpleDisplayContext::new() }
    }
} impl DisplayApi for SimpleDisplay {
    fn draw(&mut self, buffer: &[u8]) {
        crate::internal::framebuffer::with_framebuffer(|fb, _| {
            if buffer.len() != fb.len() {
                panic!("Frame buffer data does not match the expected size!");
            }

            for (i, byte) in buffer.iter().enumerate() {
                fb[i] = *byte;
            }
        }).unwrap_or_else(|| panic!("No framebuffer available when drawing to display!"));
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
        crate::internal::framebuffer::with_framebuffer(|fb, info| {
            for byte_offset in (0..fb.len()).step_by(info.bytes_per_pixel) {
                set_pixel_in_at(fb, info, byte_offset, color);
            }
        }).unwrap_or_else(|| panic!("No framebuffer available when clearing display!"));
    }

    fn swap(&mut self) { self.context.swap(); }

    fn get_info(&self) -> FrameBufferInfo {
        crate::internal::framebuffer::with_framebuffer(|_, info| info)
            .unwrap_or_else(|| panic!("No framebuffer available when getting info!"))
    }
}

pub struct BufferedDisplay {
    context: BufferedDisplayContext
} impl BufferedDisplay {
    pub fn new() -> Self {
        Self { context: BufferedDisplayContext::new() }
    }
} impl DisplayApi for BufferedDisplay {
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
        crate::internal::framebuffer::with_framebuffer(|_, info| {
            for byte_offset in (0..self.context.back_buffer.len()).step_by(info.bytes_per_pixel) {
                set_pixel_in_at(&mut self.context.back_buffer, info, byte_offset, color);
            }
        }).unwrap_or_else(|| panic!("No framebuffer available when clearing display!"));
    }

    fn swap(&mut self) { self.context.swap(); }

    fn get_info(&self) -> FrameBufferInfo {
        crate::internal::framebuffer::with_framebuffer(|_, info| info)
            .unwrap_or_else(|| panic!("No framebuffer available when getting info!"))
    }
}

struct SimpleDisplayContext;
impl DisplayContext for SimpleDisplayContext {
    fn new() -> Self { Self {} }

    fn set_pixel(&mut self, position: Position, color: Color) {
        crate::internal::framebuffer::with_framebuffer(|fb, info| {
            let byte_offset = {
                let line_offset = position.y * info.stride;
                let pixel_offset = line_offset + position.x;
                pixel_offset * info.bytes_per_pixel
            };

            set_pixel_in_at(fb, info, byte_offset, color);
        }).unwrap_or_else(|| panic!("No framebuffer available when setting pixel!"));
    }

    fn swap(&mut self) {}
} impl DrawTarget for SimpleDisplayContext {
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
} impl Dimensions for SimpleDisplayContext {
    fn bounding_box(&self) -> Rectangle {
        crate::internal::framebuffer::with_framebuffer(|_, info| {
            get_bounds(info)
        }).unwrap_or_else(|| panic!("No framebuffer available when getting bounds!"))
    }
}

struct BufferedDisplayContext {
    back_buffer: Vec<u8>,
} impl DisplayContext for BufferedDisplayContext {
    fn new() -> Self {
        let fb_len = crate::internal::framebuffer::with_framebuffer(|fb, _| {
            fb.len()
        }).unwrap_or_else(|| panic!("No framebuffer available when creating buffered display context!"));

        Self { back_buffer: vec![0; fb_len] }
    }

    fn set_pixel(&mut self, position: Position, color: Color) {
        crate::internal::framebuffer::with_framebuffer(|_, info| {
            let byte_offset = {
                let line_offset = position.y * info.stride;
                let pixel_offset = line_offset + position.x;
                pixel_offset * info.bytes_per_pixel
            };

            set_pixel_in_at(&mut self.back_buffer, info, byte_offset, color);
        }).unwrap_or_else(|| panic!("No framebuffer available when setting pixel!"));
    }

    fn swap(&mut self) {
        crate::internal::framebuffer::with_framebuffer(|fb, _| {
            let frame_buffer_len = fb.len();
            let back_buffer_len = self.back_buffer.len();

            if frame_buffer_len != back_buffer_len {
                panic!("Frame buffer and back buffer lengths do not match!");
            }

            fb.copy_from_slice(&self.back_buffer);
        }).unwrap_or_else(|| panic!("No framebuffer available when swapping display!"));
    }
} impl DrawTarget for BufferedDisplayContext {
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
} impl Dimensions for BufferedDisplayContext {
    fn bounding_box(&self) -> Rectangle {
        crate::internal::framebuffer::with_framebuffer(|_, info| {
            get_bounds(info)
        }).unwrap_or_else(|| panic!("No framebuffer available when getting bounds!"))
    }
}

fn get_bounds(info: FrameBufferInfo) -> Rectangle {
    Rectangle::new(
        Point::new(0, 0),
        embedded_graphics::geometry::Size::new(
            info.width as u32,
            info.height as u32
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
        },
        PixelFormat::Bgr => {
            pixel_buffer[0] = color.blue;
            pixel_buffer[1] = color.green;
            pixel_buffer[2] = color.red;
        },
        PixelFormat::U8 => {
            let gray = color.red / 3 + color.green / 3 + color.blue / 3;
            pixel_buffer[0] = gray;
        },
        other => panic!("Unsupported pixel format: {:?}", other)
    }
}