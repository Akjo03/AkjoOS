use alloc::sync::Arc;
use spin::Mutex;
use crate::api::time::TimeApi;
use crate::systems::time::SimpleClock;

pub struct TimeManager {
    clock: Arc<Mutex<dyn TimeApi + Send>>
} #[allow(dead_code)] impl TimeManager {
    pub fn new() -> Self {
        let clock = Arc::new(Mutex::new(SimpleClock::new()));
        crate::internal::event::EventDispatcher::global().register(clock.clone());
        Self { clock }
    }

    pub fn with_clock<F, T>(&self, f: F) -> T
        where F: FnOnce(&mut dyn TimeApi) -> T
    {
        let mut clock = self.clock.lock();
        f(&mut *clock)
    }
}