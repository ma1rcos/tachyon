use core::alloc::{GlobalAlloc, Layout};
use slab_allocator::Heap;
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
        heap.allocate(layout).unwrap_or_else(|_| core::ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Ensure `ptr` is valid before proceeding
        debug_assert!(!ptr.is_null(), "Attempted to deallocate a null pointer");

        HEAP.lock().deallocate(ptr, layout);
    }
}