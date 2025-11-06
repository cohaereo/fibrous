#[cfg(target_os = "windows")]
pub mod win32;

#[cfg(all(feature = "ucontext", target_os = "linux"))]
pub mod ucontext;
