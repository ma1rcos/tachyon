use crate::{
    memory::KernelMapper,
    paging::{mapper::PageFlushAll, Page, PageFlags, VirtualAddress},
};
use rmm::Flusher;

#[cfg(not(feature = "slab"))]
pub use self::linked_list::Allocator;

#[cfg(feature = "slab")]
pub use self::slab::Allocator;

#[cfg(not(feature = "slab"))]
mod linked_list;

#[cfg(feature = "slab")]
mod slab;

/// Maps the heap memory into the kernel's address space.
/// 
/// # Safety
/// - `offset` and `size` must be correctly aligned and within the kernel memory range.
/// - This function should only be called once during initialization.
unsafe fn map_heap(mapper: &mut KernelMapper, offset: usize, size: usize) {
    debug_assert!(size > 0, "Heap size must be greater than zero");

    if let Some(mapper) = mapper.get_mut() {
        let heap_start_page = Page::containing_address(VirtualAddress::new(offset));
        let heap_end_page = Page::containing_address(VirtualAddress::new(offset + size - 1));

        for page in Page::range_inclusive(heap_start_page, heap_end_page) {
            if let Err(e) = mapper.map(
                page.start_address(),
                PageFlags::new()
                    .write(true)
                    .global(cfg!(not(feature = "pti"))),
            ) {
                log::error!("Failed to map kernel heap: {:?}", e);
                return;
            }
        }
    } else {
        log::error!("Failed to obtain exclusive access to KernelMapper while extending heap");
    }
}

/// Initializes the kernel heap.
/// 
/// # Safety
/// - This function must be called only once during system initialization.
pub unsafe fn init() {
    let offset = crate::KERNEL_HEAP_OFFSET;
    let size = crate::KERNEL_HEAP_SIZE;

    debug_assert!(size > 0, "Heap size must be non-zero");

    // Map heap pages
    map_heap(&mut KernelMapper::lock(), offset, size);

    // Initialize global heap allocator
    Allocator::init(offset, size);
}