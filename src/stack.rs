pub struct FiberStack {
    pub base: *mut u8,
    pub size: usize,
}

impl FiberStack {
    pub fn new(size: usize) -> Self {
        allocate_fiber_stack(size)
    }
}

impl Drop for FiberStack {
    fn drop(&mut self) {
        unsafe {
            free_fiber_stack(self);
        }
    }
}

fn allocate_fiber_stack(size: usize) -> FiberStack {
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

        FiberStack { base, size }
    }
}

unsafe fn free_fiber_stack(stack: &FiberStack) {
    use std::alloc::{dealloc, Layout};

    let align = 16.max(align_of::<usize>());
    if let Ok(layout) = Layout::from_size_align(stack.size, align) {
        unsafe {
            dealloc(stack.base, layout);
        }
    }
}
