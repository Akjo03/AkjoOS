use core::sync::atomic::Ordering;
use crate::internal::event::ErrorEvent;
use crate::{KernelRuntime, Kernel};
use crate::api::display::{Colors, DisplayApi, Fonts, Position, TextAlignment, TextBaseline, TextLineHeight};
use crate::systems::display::SimpleDisplay;

impl KernelRuntime for Kernel {
    fn init(&mut self) {}

    fn tick(&mut self) {
        crate::internal::framebuffer::with_framebuffer(|fb, fb_info| {
            let mut display = SimpleDisplay::new(fb, fb_info);

            display.clear(Colors::Black.into());
            display.draw_text(
                "Hello World!", Position::new(0, 0),
                Colors::White.into(), None,
                Fonts::default().into(), false, false,
                TextBaseline::Top, TextAlignment::Left, TextLineHeight::Full
            );
        });
        self.running.store(false, Ordering::SeqCst);
    }

    fn on_error(&mut self, _event: ErrorEvent) {}

    fn halt(&mut self) {}
}