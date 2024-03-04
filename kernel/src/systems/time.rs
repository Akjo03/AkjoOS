use crate::api::time::{DateTime, Month, TimeApi, TimeOffset};
use crate::api::event::{Event, EventHandler};

pub struct SimpleClock {
    current_time: DateTime
} impl SimpleClock {
    pub fn new() -> Self { Self {
        current_time: DateTime::new(0, 0, 0, 0, 1, Month::January, 1970)
    } }
} impl TimeApi for SimpleClock {
    fn now(&self) -> DateTime {
        self.current_time.clone()
    }

    fn with_offset(&self, offset: TimeOffset) -> DateTime {
        self.current_time.with_offset(offset).clone()
    }
} impl EventHandler for SimpleClock {
    fn handle(&mut self, event: Event) {
        match event {
            Event::Rtc(date_time) => self.current_time = DateTime::from_rtc(date_time),
            _ => {}
        }
    }
}