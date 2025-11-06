use crate::FiberApi;

pub struct UContextFiberApi;

unsafe impl FiberApi for UContextFiberApi {
    unsafe fn create_fiber(
        stack_size: usize,
        entry: crate::FiberEntry,
        user_data: *mut (),
    ) -> Result<crate::FiberHandle, crate::FiberError> {
        todo!()
    }

    unsafe fn convert_thread_to_fiber() -> Result<crate::FiberHandle, crate::FiberError> {
        todo!()
    }

    unsafe fn switch_to_fiber(to: crate::FiberHandle) {
        todo!()
    }

    unsafe fn destroy_fiber(handle: crate::FiberHandle) {
        todo!()
    }
}
