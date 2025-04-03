use core::{fmt, ptr::NonNull};

/// A wrapper around `NonNull<T>` that is `Send` + `Sync`, ensuring the pointer remains unique.
///
/// # Safety
///
/// This type must only be used when the pointer is uniquely owned and never accessed concurrently.
#[repr(transparent)]
pub struct Unique<T: ?Sized>(NonNull<T>);

unsafe impl<T: ?Sized> Send for Unique<T> {}
unsafe impl<T: ?Sized> Sync for Unique<T> {}

impl<T: ?Sized> Unique<T> {
    /// Creates a new `Unique<T>` from a raw pointer **without checking for null**.
    ///
    /// # Safety
    ///
    /// The caller **must** ensure that `ptr` is:
    /// - Non-null
    /// - Properly aligned
    /// - Pointing to a valid instance of `T`
    #[must_use]
    #[inline]
    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        debug_assert!(!ptr.is_null(), "Attempted to create Unique<T> from a null pointer");
        Self(NonNull::new_unchecked(ptr))
    }

    /// Returns the raw pointer contained within `Unique<T>`.
    #[must_use]
    #[inline]
    pub fn as_ptr(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T: ?Sized> Clone for Unique<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> fmt::Debug for Unique<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unique({:p})", self.0.as_ptr())
    }
}