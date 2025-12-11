/// Represents a valid fiber stack base pointer and size.
#[derive(Debug, Clone, Copy)]
pub struct FiberStackPointer {
    base: *mut u8,
    size: usize,
    _private: (),
}

impl FiberStackPointer {
    /// Create a FiberStack from a raw base pointer and size.
    ///
    /// **⚠️ Panics if `base` is null or not 16-byte aligned.**
    ///
    /// # Safety
    /// - The memory region pointed to by `base` must be valid for the duration of the FiberStack's lifetime.
    /// - The same memory region must be at least `size` bytes long.
    pub unsafe fn from_base_size(base: *mut u8, size: usize) -> Self {
        assert!(!base.is_null(), "Fiber stack base pointer cannot be null");
        assert_eq!(
            base.align_offset(16),
            0,
            "Fiber stack base pointer must be 16-byte aligned"
        );
        Self {
            base,
            size,
            _private: (),
        }
    }

    pub fn base(&self) -> *mut u8 {
        self.base
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct FiberStack(FiberStackPointer);

impl FiberStack {
    /// Allocate a new fiber stack with the given size.
    ///
    /// # Safety
    /// - The caller is responsible for freeing the allocated memory using `FiberStack::free`.
    pub fn new(size: usize) -> Self {
        use std::alloc::{alloc, handle_alloc_error, Layout};

        // Ensure minimum size and alignment
        let size = size.max(64 * 1024).next_multiple_of(4096);
        let align = 16.max(align_of::<usize>());

        let layout = Layout::from_size_align(size, align).expect("Invalid layout for fiber stack");

        unsafe {
            let base = alloc(layout);
            if base.is_null() {
                handle_alloc_error(layout);
            }

            Self(FiberStackPointer::from_base_size(base, size))
        }
    }
    
    pub fn as_pointer(&self) -> FiberStackPointer {
        self.0
    }
}

impl Drop for FiberStack {
    fn drop(&mut self) {
        use std::alloc::{dealloc, Layout};

        let align = 16.max(align_of::<usize>());
        if let Ok(layout) = Layout::from_size_align(self.0.size, align) {
            unsafe {
                dealloc(self.0.base, layout);
            }
        }
    }
}
