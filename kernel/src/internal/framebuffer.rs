use bootloader_api::info::FrameBufferInfo;
use spin::Lazy;
use spin::lock_api::Mutex;

static FRAMEBUFFER: Lazy<Mutex<Option<&'static mut [u8]>>> = Lazy::new(|| {
    Mutex::new(None)
});

static FRAMEBUFFER_INFO: Lazy<Mutex<Option<FrameBufferInfo>>> = Lazy::new(|| {
    Mutex::new(None)
});

pub fn init(frame_buffer_info: FrameBufferInfo, frame_buffer: &'static mut [u8]) {
    let mut fb_guard = FRAMEBUFFER.lock();
    *fb_guard = Some(frame_buffer);

    let mut info_guard = FRAMEBUFFER_INFO.lock();
    *info_guard = Some(frame_buffer_info);
}

pub fn with_framebuffer<F, R>(func: F) -> Option<R>
    where F: FnOnce(&mut [u8], FrameBufferInfo) -> R {

    let mut fb_guard = FRAMEBUFFER.lock();
    let info_guard = FRAMEBUFFER_INFO.lock();

    if let (Some(fb), Some(info)) = (&mut *fb_guard, &*info_guard) {
        Some(func(fb, *info))
    } else { None }
}

pub fn is_initialized() -> bool {
    let fb_guard = FRAMEBUFFER.lock();
    let info_guard = FRAMEBUFFER_INFO.lock();

    fb_guard.is_some() && info_guard.is_some()
}