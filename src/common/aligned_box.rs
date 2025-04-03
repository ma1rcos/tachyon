use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, NonNull};

use crate::{common::unique::Unique, memory::Enomem};

// Ensures that the type is safe for zero-initialization
pub unsafe trait ValidForZero {}
unsafe impl<const N: usize> ValidForZero for [u8; N] {}
unsafe impl ValidForZero for u8 {}

/// A heap-allocated, aligned structure.
/// Ensures that memory is allocated with the specified alignment.
pub struct AlignedBox<T: ?Sized, const ALIGN: usize> {
    inner: Unique<T>,
}

impl<T: ?Sized, const ALIGN: usize> AlignedBox<T, ALIGN> {
    /// Returns the memory layout for this allocation.
    fn layout(&self) -> Layout {
        layout_with_align(Layout::for_value(&*self), ALIGN)
    }
}

/// Adjusts the alignment of a given memory layout, ensuring it is valid.
const fn layout_with_align(layout: Layout, align: usize) -> Layout {
    match Layout::from_size_align(layout.size(), align.max(layout.align())) {
        Ok(l) => l,
        Err(_) => panic!("Invalid memory layout requested"),
    }
}

impl<T, const ALIGN: usize> AlignedBox<T, ALIGN> {
    /// Tries to allocate and zero-initialize an aligned box.
    #[inline(always)]
    pub fn try_zeroed() -> Result<Self, Enomem>
    where
        T: ValidForZero,
    {
        let layout = layout_with_align(Layout::new::<T>(), ALIGN);
        let ptr = unsafe { crate::ALLOCATOR.alloc_zeroed(layout) };

        NonNull::new(ptr.cast()).map_or_else(|| Err(Enomem), |p| Ok(Self { inner: Unique::new_unchecked(p) }))
    }
}

impl<T, const ALIGN: usize> AlignedBox<[T], ALIGN> {
    /// Tries to allocate and zero-initialize a slice of elements.
    #[inline]
    pub fn try_zeroed_slice(len: usize) -> Result<Self, Enomem>
    where
        T: ValidForZero,
    {
        let layout = Layout::array::<T>(len).ok_or(Enomem)?;
        let aligned_layout = layout_with_align(layout, ALIGN);
        let ptr = unsafe { crate::ALLOCATOR.alloc_zeroed(aligned_layout) };

        NonNull::new(ptr.cast()).map_or_else(|| Err(Enomem), |p| {
            Ok(Self {
                inner: Unique::new_unchecked(ptr::slice_from_raw_parts_mut(p.as_ptr(), len)),
            })
        })
    }
}

impl<T: ?Sized, const ALIGN: usize> core::fmt::Debug for AlignedBox<T, ALIGN> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[AlignedBox at {:p}, size: {}, alignment: {}]",
            self.inner.as_ptr(),
            self.layout().size(),
            self.layout().align()
        )
    }
}

impl<T: ?Sized, const ALIGN: usize> Drop for AlignedBox<T, ALIGN> {
    fn drop(&mut self) {
        unsafe {
            let layout = self.layout();
            ptr::drop_in_place(self.inner.as_ptr());
            crate::ALLOCATOR.dealloc(self.inner.as_ptr().cast(), layout);
        }
    }
}

impl<T: ?Sized, const ALIGN: usize> core::ops::Deref for AlignedBox<T, ALIGN> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.as_ptr() }
    }
}

impl<T: ?Sized, const ALIGN: usize> core::ops::DerefMut for AlignedBox<T, ALIGN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.as_ptr() }
    }
}

impl<T: Clone + ValidForZero, const ALIGN: usize> Clone for AlignedBox<T, ALIGN> {
    fn clone(&self) -> Self {
        let mut new = Self::try_zeroed().unwrap_or_else(|_| alloc::alloc::handle_alloc_error(self.layout()));
        T::clone_from(&mut new, self);
        new
    }
}

impl<T: Clone + ValidForZero, const ALIGN: usize> Clone for AlignedBox<[T], ALIGN> {
    fn clone(&self) -> Self {
        if self.len() == 0 {
            return Self::try_zeroed_slice(0).unwrap();
        }

        let mut new = Self::try_zeroed_slice(self.len())
            .unwrap_or_else(|_| alloc::alloc::handle_alloc_error(self.layout()));

        for (dst, src) in new.iter_mut().zip(self.iter()) {
            dst.clone_from(src);
        }

        new
    }
}