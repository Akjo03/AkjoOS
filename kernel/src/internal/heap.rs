use alloc::collections::VecDeque;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use bootloader_api::info::MemoryRegions;
use linked_list_allocator::LockedHeap;
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;

pub const INITIAL_HEAP_START: usize = 0x_1111_1111_0000;
pub const INITIAL_HEAP_SIZE: usize = 1024 * 1024 * 2; // 2 MiB

pub const MAIN_HEAP_START: usize = 0x_4444_4444_0000;
pub const MAIN_HEAP_SIZE: usize = 1024 * 1024 * 64; // 64 MiB

#[global_allocator]
static ALLOCATOR: HeapManager = HeapManager::new();

pub fn init_allocator() {
    ALLOCATOR.init();
}

pub struct HeapManager {
    initial_heap: LockedHeap,
    main_heap: LockedHeap,
    initialized: AtomicBool,
} impl HeapManager {
    const fn new() -> Self { Self {
        initial_heap: LockedHeap::empty(),
        main_heap: LockedHeap::empty(),
        initialized: AtomicBool::new(false),
    } }

    unsafe fn init_initial_heap(&self, start: usize, size: usize) {
        self.initial_heap.lock().init(start as *mut u8, size);
    }

    unsafe fn init_main_heap(&self, start: usize, size: usize) {
        self.main_heap.lock().init(start as *mut u8, size);
    }

    fn init(&self) {
        self.initialized.store(true, Ordering::SeqCst);
    }
} unsafe impl GlobalAlloc for HeapManager {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.initialized.load(Ordering::SeqCst) {
            self.main_heap.alloc(layout)
        } else {
            self.initial_heap.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.initialized.load(Ordering::SeqCst) {
            self.main_heap.dealloc(ptr, layout)
        } else {
            self.initial_heap.dealloc(ptr, layout)
        }
    }
}

pub struct SimpleHeapFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
} impl SimpleHeapFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions, next: usize) -> Self { Self {
        memory_regions, next
    } }

    pub fn usable_regions(&self) -> impl Iterator<Item = PhysFrame> {
        crate::internal::memory::get_usable_regions(self.memory_regions, self.next)
    }
} unsafe impl FrameAllocator<Size4KiB> for SimpleHeapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_regions().next();
        self.next += 1;
        frame
    }
}

pub struct HeapFrameAllocator {
    usable_frames: VecDeque<PhysFrame>,
    next: usize,
} impl HeapFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions, next: usize) -> Self {
        let usable_frames: VecDeque<_> = crate::internal::memory::get_usable_regions(memory_regions, next).collect();
        Self { next, usable_frames }
    }
} unsafe impl FrameAllocator<Size4KiB> for HeapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next += 1;
        self.usable_frames.pop_front()
    }
}

pub fn init_initial_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut SimpleHeapFrameAllocator,
) -> Result<usize, MapToError<Size4KiB>> {
    init_heap_range(mapper, frame_allocator, INITIAL_HEAP_START, INITIAL_HEAP_SIZE)?;

    unsafe { ALLOCATOR.init_initial_heap(INITIAL_HEAP_START, INITIAL_HEAP_SIZE); }

    Ok(frame_allocator.next)
}

pub fn init_main_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut HeapFrameAllocator,
) -> Result<usize, MapToError<Size4KiB>> {
    init_heap_range(mapper, frame_allocator, MAIN_HEAP_START, MAIN_HEAP_SIZE)?;

    unsafe { ALLOCATOR.init_main_heap(MAIN_HEAP_START, MAIN_HEAP_SIZE); }

    Ok(frame_allocator.next)
}

fn init_heap_range(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    start: usize,
    size: usize,
) -> Result<(), MapToError<Size4KiB>> {
    let initial_page_range = {
        let initial_heap_start = VirtAddr::new(start as u64);
        let initial_heap_end = initial_heap_start + size - 1u64;
        let initial_heap_start_page = Page::containing_address(initial_heap_start);
        let initial_heap_end_page = Page::containing_address(initial_heap_end);
        Page::range_inclusive(initial_heap_start_page, initial_heap_end_page)
    };

    for page in initial_page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    log::info!("Initialized heap range: {:#x?} - {:#x?}", start, start + size);

    Ok(())
}