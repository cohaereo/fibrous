use crate::{stack::FiberStackPointer, FiberApi, FiberEntry, FiberHandle};

struct LibcFiberContext {
    ucp: libc::ucontext_t,
    stack: Option<FiberStackPointer>,
}

impl Default for LibcFiberContext {
    fn default() -> Self {
        LibcFiberContext {
            ucp: unsafe { std::mem::zeroed() },
            stack: None,
        }
    }
}

extern "C" fn fiber_wrapper(entry: FiberEntry, user_data: *mut ()) {
    unsafe {
        entry(user_data);

        std::process::abort();
    }
}

pub struct UContextFiberApi;

unsafe impl FiberApi for UContextFiberApi {
    unsafe fn create_fiber(
        stack: FiberStackPointer,
        entry: crate::FiberEntry,
        user_data: *mut (),
    ) -> Result<FiberHandle, crate::FiberError> {
        let mut ctx = Box::new(LibcFiberContext::default());

        if libc::getcontext(&mut ctx.ucp) != 0 {
            return Err(crate::FiberError::CreationFailed);
        }

        ctx.ucp.uc_stack.ss_sp = stack.base() as *mut libc::c_void;
        ctx.ucp.uc_stack.ss_size = stack.size();
        ctx.ucp.uc_link = std::ptr::null_mut();

        ctx.stack = Some(stack);

        libc::makecontext(
            &raw mut ctx.ucp,
            std::mem::transmute::<extern "C" fn(_, _), extern "C" fn()>(fiber_wrapper),
            2,
            entry,
            user_data,
        );

        Ok(FiberHandle(Box::into_raw(ctx) as *mut ()))
    }

    unsafe fn convert_thread_to_fiber() -> Result<FiberHandle, crate::FiberError> {
        let mut ctx = Box::new(LibcFiberContext::default());
        if libc::getcontext(&mut ctx.ucp) != 0 {
            return Err(crate::FiberError::CreationFailed);
        }

        let raw = Box::into_raw(ctx);
        Ok(FiberHandle(raw as *mut ()))
    }

    unsafe fn switch_to_fiber(from: FiberHandle, to: FiberHandle) {
        let from_ctx = &mut *(from.0 as *mut LibcFiberContext);
        let to_ctx = &mut *(to.0 as *mut LibcFiberContext);
        libc::swapcontext(&mut from_ctx.ucp, &to_ctx.ucp);
    }

    unsafe fn destroy_fiber(handle: FiberHandle) {
        let _ctx = Box::from_raw(handle.0 as *mut LibcFiberContext);
    }
}
