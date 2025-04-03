use crate::memory::KernelMapper;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};
use linked_list_allocator::Heap;
use spin::Mutex;

/// Global heap allocator, protected by a spinlock.
static HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

pub struct Allocator;

impl Allocator {
    /// Initializes the heap with the given offset and size.
    /// Safety: This function must be called only once before any allocation.
    pub unsafe fn init(offset: usize, size: usize) {
        HEAP.lock().init(offset, size);
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = HEAP.lock();
        match heap.allocate_first_fit(layout) {
            Ok(allocation) => allocation.as_ptr(),
            Err(()) => {
                // Get the current heap size and extend if possible
                let size = heap.size();
                super::map_heap(
                    &mut KernelMapper::lock(),
                    crate::KERNEL_HEAP_OFFSET + size,
                    crate::KERNEL_HEAP_SIZE,
                );
                heap.extend(crate::KERNEL_HEAP_SIZE);

                // Retry allocation after extending the heap
                heap.allocate_first_fit(layout)
                    .ok()
                    .map_or(ptr::null_mut(), |allocation| allocation.as_ptr())
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Ensure `ptr` is valid before proceeding
        debug_assert!(!ptr.is_null(), "Attempted to deallocate a null pointer");

        let mut heap = HEAP.lock();
        heap.deallocate(NonNull::new_unchecked(ptr), layout);
    }
}