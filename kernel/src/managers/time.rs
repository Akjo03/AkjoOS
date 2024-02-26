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

    pub fn with_clock<F, T>(&self, func: F) -> Option<T>
        where F: FnOnce(&mut dyn TimeApi) -> T
    {
        if let Some(mut clock) = self.clock.try_lock() {
            Some(func(&mut *clock))
        } else { None }
    }
}