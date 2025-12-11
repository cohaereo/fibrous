use std::{error::Error, fmt::Display};

use cfg_if::cfg_if;
pub use crate::stack::{FiberStackPointer, FiberStack};

mod stack;

pub mod r#impl;

cfg_if! {
    if #[cfg(all(feature = "ucontext", target_os = "linux"))] {
        pub type DefaultFiberApi = r#impl::ucontext::UContextFiberApi;
    } else {
        compile_error!("Unsupported platform");
    }
}

/// Raw fiber handle - platform-specific opaque pointer
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiberHandle(*mut ());

impl FiberHandle {
    pub const fn null() -> Self {
        FiberHandle(std::ptr::null_mut())
    }
}

unsafe impl Send for FiberHandle {}

pub type FiberEntry = unsafe extern "C" fn(*mut ());

/// # Safety
/// This trait defines low-level fiber operations that are inherently unsafe. Follow each method's safety notes carefully.
pub unsafe trait FiberApi {
    /// Create a new fiber with the given stack and entry point
    ///
    /// # Safety
    /// - `stack` must be valid for the lifetime of the fiber
    /// - `stack.size` must be sufficient for the platform to prevent stack overflows (typically >= 64KB)
    /// - `entry` must be a valid function pointer
    /// - `entry` must never return (must switch to another fiber or exit)
    unsafe fn create_fiber(
        stack: FiberStackPointer,
        entry: FiberEntry,
        user_data: *mut (),
    ) -> Result<FiberHandle, FiberError>;

    /// Convert the current thread into a fiber
    /// This must be called before any fiber switching can occur
    ///
    /// # Safety
    /// - Must be called exactly once per thread
    /// - Must be called before any `switch_to_fiber` calls
    unsafe fn convert_thread_to_fiber() -> Result<FiberHandle, FiberError>;

    /// Switch execution from the current fiber to the target fiber
    ///
    /// # Safety
    /// - `from` must be the currently executing fiber
    /// - `to` must be a valid fiber handle
    /// - Both fibers must be on the same thread (no fiber migration)
    /// - Caller must ensure proper synchronization of shared state
    unsafe fn switch_to_fiber(from: FiberHandle, to: FiberHandle);

    /// Destroy a fiber and free its resources
    ///
    /// # Safety
    /// - `handle` must be a valid fiber that is not currently executing
    /// - Must not be called on the main thread fiber
    /// - The fiber must not be switched to after this call
    unsafe fn destroy_fiber(handle: FiberHandle);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiberError {
    /// Unspecified error during fiber creation
    CreationFailed,
    PlatformError(i32),
}

impl Error for FiberError {}

impl Display for FiberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FiberError::CreationFailed => write!(f, "Fiber creation failed"),
            FiberError::PlatformError(code) => write!(f, "Platform error with code {}", code),
        }
    }
}
