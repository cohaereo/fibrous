
/// Helper to allocate a fiber stack with proper alignment
pub fn allocate_fiber_stack(size: usize) -> FiberStack {
    use std::alloc::{Layout, alloc, handle_alloc_error};

    // Ensure minimum size and alignment
    let size = size.max(64 * 1024).next_multiple_of(4096);
    let align = 16.max(std::mem::align_of::<usize>());

    let layout = Layout::from_size_align(size, align).expect("Invalid layout for fiber stack");

    unsafe {
        let base = alloc(layout);
        if base.is_null() {
            handle_alloc_error(layout);
        }

        FiberStack { base, size }
    }
}

/// Helper to free a fiber stack
pub unsafe fn free_fiber_stack(stack: FiberStack) {
    use std::alloc::{Layout, dealloc};

    let align = 16.max(std::mem::align_of::<usize>());
    if let Ok(layout) = Layout::from_size_align(stack.size, align) {
        unsafe {
            dealloc(stack.base, layout);
        }
    }
}
