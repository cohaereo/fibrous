use crate::{FiberApi, FiberHandle};

unsafe extern "stdcall" {
    fn ConvertThreadToFiber(lpParameter: *mut ()) -> *mut ();
    fn CreateFiberEx(
        dwStackCommitSize: usize,
        dwStackReserveSize: usize,
        dwFlags: u32,
        lpStartAddress: unsafe extern "C" fn(*mut ()),
        lpParameter: *mut (),
    ) -> *mut ();
    fn SwitchToFiber(lpFiber: *mut ());
    fn DeleteFiber(lpFiber: *mut ());
    fn GetLastError() -> i32;
}

const FIBER_FLAG_FLOAT_SWITCH: u32 = 0x0001;

pub struct Win32FiberApi;

unsafe impl FiberApi for Win32FiberApi {
    unsafe fn create_fiber(
        stack_size: usize,
        entry: crate::FiberEntry,
        user_data: *mut (),
    ) -> Result<FiberHandle, crate::FiberError> {
        // TODO(cohae): Floating point handling is default by now but may become optional in the future
        let fiber_ptr = CreateFiberEx(stack_size, 0, FIBER_FLAG_FLOAT_SWITCH, entry, user_data);
        if fiber_ptr.is_null() {
            return Err(crate::FiberError::PlatformError(GetLastError()));
        }
        Ok(FiberHandle(fiber_ptr))
    }

    unsafe fn convert_thread_to_fiber() -> Result<FiberHandle, crate::FiberError> {
        let fiber_ptr = ConvertThreadToFiber(std::ptr::null_mut());
        if fiber_ptr.is_null() {
            return Err(crate::FiberError::PlatformError(GetLastError()));
        }
        Ok(FiberHandle(fiber_ptr))
    }

    unsafe fn switch_to_fiber(to: FiberHandle) {
        SwitchToFiber(to.0);
    }

    unsafe fn destroy_fiber(handle: FiberHandle) {
        DeleteFiber(handle.0);
    }
}
