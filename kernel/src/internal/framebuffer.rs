use alloc::sync::Arc;
use bootloader_api::info::FrameBufferInfo;
use spin::{Mutex, Once};

static FRAMEBUFFER: Once<Arc<Mutex<&'static mut [u8]>>> = Once::new();
static FRAMEBUFFER_INFO: Once<FrameBufferInfo> = Once::new();

pub struct Framebuffer {
    buffer: Arc<Mutex<&'static mut [u8]>>,
    info: FrameBufferInfo,
} impl Framebuffer {
    pub(crate) fn global() -> Option<Self> {
        Some(Self {
            buffer: FRAMEBUFFER.get()?.clone(),
            info: *FRAMEBUFFER_INFO.get()?,
        })
    }

    pub fn consume<F, T>(&self, func: F) -> Option<T>
        where F: FnOnce(&mut [u8], FrameBufferInfo) -> T, {

        if let Some(mut buffer) = self.buffer.try_lock() {
            Some(func(&mut buffer, self.info))
        } else { None }
    }
}

pub fn init(buffer: &'static mut [u8], info: FrameBufferInfo) {
    FRAMEBUFFER.call_once(|| Arc::new(Mutex::new(buffer)));
    FRAMEBUFFER_INFO.call_once(|| info);
}