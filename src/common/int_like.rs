#[macro_export]
macro_rules! int_like {
    // Define a single integer-backed type
    ($new_type_name:ident, $backing_type:ty) => {
        #[repr(transparent)]
        #[derive(Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
        pub struct $new_type_name($backing_type);

        impl $new_type_name {
            /// Returns the underlying value.
            #[must_use]
            #[inline]
            pub const fn get(self) -> $backing_type {
                self.0
            }

            /// Creates a new instance from a raw integer value.
            #[must_use]
            #[inline]
            pub const fn new(x: $backing_type) -> Self {
                Self(x)
            }
        }

        impl ::core::convert::From<$backing_type> for $new_type_name {
            #[inline]
            fn from(inner: $backing_type) -> Self {
                Self(inner)
            }
        }

        impl ::core::convert::From<$new_type_name> for $backing_type {
            #[inline]
            fn from(wrapped: $new_type_name) -> Self {
                wrapped.get()
            }
        }
    };

    // Define an integer-backed type and its atomic counterpart
    ($new_type_name:ident, $new_atomic_type_name:ident, $backing_type:ty, $backing_atomic_type:ty) => {
        int_like!($new_type_name, $backing_type);

        /// A thread-safe atomic wrapper for `$new_type_name`.
        #[repr(transparent)]
        pub struct $new_atomic_type_name {
            container: $backing_atomic_type,
        }

        impl $new_atomic_type_name {
            /// Creates a new atomic instance.
            #[must_use]
            #[inline]
            pub const fn new(x: $new_type_name) -> Self {
                Self {
                    container: $backing_atomic_type::new(x.get()),
                }
            }

            /// Atomically loads the value.
            #[must_use]
            #[inline]
            pub fn load(&self, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.load(order))
            }

            /// Atomically stores a new value.
            #[inline]
            pub fn store(&self, val: $new_type_name, order: ::core::sync::atomic::Ordering) {
                self.container.store(val.get(), order);
            }

            /// Atomically swaps the value.
            #[must_use]
            #[inline]
            pub fn swap(&self, val: $new_type_name, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.swap(val.get(), order))
            }

            /// Atomically increments the value.
            #[must_use]
            #[inline]
            pub fn fetch_add(&self, with: $new_type_name, order: ::core::sync::atomic::Ordering) -> $new_type_name {
                $new_type_name::from(self.container.fetch_add(with.get(), order))
            }

            /// Atomically compares and swaps values.
            #[inline]
            pub fn compare_exchange(
                &self,
                current: $new_type_name,
                new: $new_type_name,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$new_type_name, $new_type_name> {
                self.container
                    .compare_exchange(current.get(), new.get(), success, failure)
                    .map($new_type_name::from)
                    .map_err($new_type_name::from)
            }

            /// A weaker version of `compare_exchange`, that might fail spuriously.
            #[inline]
            pub fn compare_exchange_weak(
                &self,
                current: $new_type_name,
                new: $new_type_name,
                success: ::core::sync::atomic::Ordering,
                failure: ::core::sync::atomic::Ordering,
            ) -> Result<$new_type_name, $new_type_name> {
                self.container
                    .compare_exchange_weak(current.get(), new.get(), success, failure)
                    .map($new_type_name::from)
                    .map_err($new_type_name::from)
            }
        }

        impl ::core::default::Default for $new_atomic_type_name {
            #[inline]
            fn default() -> Self {
                Self::new($new_type_name::new(0))
            }
        }
    };
}

// ---------- TESTS ----------
#[test]
fn test_int_like() {
    use ::core::sync::atomic::AtomicUsize;
    use core::mem::size_of;

    // Generate a basic integer-like type.
    int_like!(UsizeLike, usize);
    assert_eq!(size_of::<UsizeLike>(), size_of::<usize>());

    // Generate an integer-like type with atomic support.
    int_like!(UsizeLike2, AtomicUsizeLike, usize, AtomicUsize);
    assert_eq!(size_of::<UsizeLike2>(), size_of::<usize>());
    assert_eq!(size_of::<AtomicUsizeLike>(), size_of::<AtomicUsize>());
}