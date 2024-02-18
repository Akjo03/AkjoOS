use alloc::collections::VecDeque;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicBool, Ordering};
use linked_list_allocator::LockedHeap;
use x86_64::{
    PhysAddr,
    structures::paging::{
        PageTable, FrameAllocator, OffsetPageTable, PhysFrame,
        mapper::MapToError, Mapper, Page, PageTableFlags
    },
    structures::paging::page::Size4KiB,
    VirtAddr
};

pub struct SimpleBootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
} impl SimpleBootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions) -> Self { Self {
        memory_regions, next: 0,
    } }

    pub fn usable_regions(&self) -> impl Iterator<Item = PhysFrame> {
        get_usable_regions(self.memory_regions, self.next)
    }
} unsafe impl FrameAllocator<Size4KiB> for SimpleBootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_regions().next();
        self.next += 1;
        frame
    }
}

pub struct BootInfoFrameAllocator {
    usable_frames: VecDeque<PhysFrame>,
    next: usize,
} impl BootInfoFrameAllocator {
    pub unsafe fn new(memory_regions: &'static MemoryRegions, next: usize) -> Self {
        let usable_frames: VecDeque<_> = get_usable_regions(memory_regions, next).collect();
        Self { next, usable_frames }
    }
} unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next += 1;
        self.usable_frames.pop_front()
    }
}

pub fn get_usable_regions(memory_regions: &'static MemoryRegions, skip: usize) -> impl Iterator<Item = PhysFrame> {
    memory_regions.iter()
        .filter(|region| region.kind == MemoryRegionKind::Usable)
        .filter(|region| region.start % 4096 == 0 && region.end % 4096 == 0)
        .map(|region| region.start..region.end)
        .flat_map(|region_range| region_range.step_by(4096))
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
        .skip(skip)
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub const INITIAL_HEAP_START: usize = 0x_1111_1111_0000;
pub const INITIAL_HEAP_SIZE: usize = 1024 * 1024 * 1; // 1 MiB

pub const MAIN_HEAP_START: usize = 0x_4444_4444_0000;
pub const MAIN_HEAP_SIZE: usize = 1024 * 1024 * 64; // 64 MiB


struct HeapManager {
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

#[global_allocator]
static ALLOCATOR: HeapManager = HeapManager::new();

pub fn init_initial_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut SimpleBootInfoFrameAllocator,
) -> Result<usize, MapToError<Size4KiB>> {
    init_heap_range(mapper, frame_allocator, INITIAL_HEAP_START, INITIAL_HEAP_SIZE)?;

    unsafe { ALLOCATOR.init_initial_heap(INITIAL_HEAP_START, INITIAL_HEAP_SIZE); }

    Ok(frame_allocator.next)
}

pub fn init_main_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut BootInfoFrameAllocator,
) -> Result<usize, MapToError<Size4KiB>> {
    init_heap_range(mapper, frame_allocator, MAIN_HEAP_START, MAIN_HEAP_SIZE)?;

    unsafe { ALLOCATOR.init_main_heap(MAIN_HEAP_START, MAIN_HEAP_SIZE); }

    Ok(frame_allocator.next)
}

pub fn init_allocator() {
    ALLOCATOR.init();
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

    Ok(())
}