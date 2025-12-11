use crate::{FiberApi, FiberEntry, FiberError, FiberHandle, FiberStackPointer};
use context::stack::Stack;
use context::{Context, Transfer};
use std::ptr;
use std::ptr::NonNull;

struct BoostFiberContext {
    /// The boost context handle. wrapped in Option so we can 'take' it
    /// when resuming (since resume consumes the struct).
    ctx: Option<Context>,
    /// Pointer to the context that switched to us.
    /// Used primarily during the first entry to the fiber to save the parent's state.
    parent: Option<NonNull<BoostFiberContext>>,
    entry: FiberEntry,
    user_data: *mut (),
}

impl BoostFiberContext {
    /// # Safety
    /// - `handle` must be a valid fiber handle created by this API
    unsafe fn from_handle(handle: FiberHandle) -> &'static mut Self {
        &mut *(handle.0 as *mut BoostFiberContext)
    }
}

extern "C" fn fcontext_fiber_wrapper(t: Transfer) -> ! {
    unsafe {
        // `t.data` is passed from `resume`. We pass the handle to the fiber being started (self).
        let self_ctx = BoostFiberContext::from_handle(FiberHandle(t.data as *mut ()));

        // `t.context` represents the state of the fiber that just suspended (the caller/parent).
        // We must save this into the parent's struct so we can switch back to it later.
        if let Some(mut parent_ptr) = self_ctx.parent {
            let parent_ctx = parent_ptr.as_mut();
            parent_ctx.ctx = Some(t.context);
        }

        (self_ctx.entry)(self_ctx.user_data);

        // Fibers should never return from their entry point
        std::process::abort();
    }
}

extern "C" fn thread_stub(_p: *mut ()) {
}

pub struct FContextFiberApi;

unsafe impl FiberApi for FContextFiberApi {
    unsafe fn create_fiber(
        stack: FiberStackPointer,
        entry: FiberEntry,
        user_data: *mut (),
    ) -> Result<FiberHandle, FiberError> {
        let stack = Stack::new(
            stack.base().add(stack.size()).cast(), // Stack top
            stack.base().cast(), // Stack bottom
        );

        let ctx = Box::new(BoostFiberContext {
            ctx: Some(Context::new(&stack, fcontext_fiber_wrapper)),
            parent: None,
            entry,
            user_data,
        });

        Ok(FiberHandle(Box::into_raw(ctx) as *mut ()))
    }

    unsafe fn convert_thread_to_fiber() -> Result<FiberHandle, FiberError> {
        let ctx = Box::new(BoostFiberContext {
            ctx: None,
            parent: None,
            entry: thread_stub, // Dummy entry point, never actually used for the main thread
            user_data: ptr::null_mut(),
        });

        Ok(FiberHandle(Box::into_raw(ctx) as *mut ()))
    }

    unsafe fn switch_to_fiber(from: FiberHandle, to: FiberHandle) {
        let from_ctx = BoostFiberContext::from_handle(from);
        let to_ctx = BoostFiberContext::from_handle(to);

        // 1. Link 'from' as the parent of 'to'.
        // If 'to' is running for the first time, the wrapper will use this to save 'from's state.
        to_ctx.parent = Some(NonNull::new_unchecked(from_ctx as *mut BoostFiberContext));

        // 2. Retrieve the target context.
        // Note: If 'to' is the main thread, it must have been populated by a previous switch
        // (the wrapper or the return path below).
        let ctx = to_ctx.ctx.take().expect("Target fiber has no valid context to resume");

        // 3. Suspend current fiber ('from') and jump to 'to'.
        // We pass 'to' (the handle pointer) as data so the wrapper can find itself.
        let transfer = ctx.resume(to.0 as usize);

        // 4. We are back in 'from'.
        // 'transfer' contains the updated state of the fiber that just suspended (which is 'to').
        // We save it back into 'to's struct so we can resume it later.
        to_ctx.ctx = Some(transfer.context);
    }

    unsafe fn destroy_fiber(handle: FiberHandle) {
        let _ctx = Box::from_raw(handle.0 as *mut BoostFiberContext);
    }
}