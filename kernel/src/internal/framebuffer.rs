use alloc::sync::Arc;
use bootloader_api::info::FrameBufferInfo;
use spin::lock_api::Mutex;
use spin::Once;

static FRAMEBUFFER: Once<Arc<Mutex<&'static mut [u8]>>> = Once::new();
static FRAMEBUFFER_INFO: Once<FrameBufferInfo> = Once::new();

pub fn init(framebuffer: &'static mut [u8], framebuffer_info: FrameBufferInfo) {
    FRAMEBUFFER.call_once(|| Arc::new(Mutex::new(framebuffer)));
    FRAMEBUFFER_INFO.call_once(|| framebuffer_info);
}