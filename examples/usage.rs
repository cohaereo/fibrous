use fibrous::{DefaultFiberApi, FiberApi, FiberHandle, FiberStack};

const STACK_SIZE: usize = 4 * 1024; // 4 KB

static mut MAIN_FIBER: FiberHandle = FiberHandle::null();
static mut SECOND_FIBER: FiberHandle = FiberHandle::null();

fn main() {
    std::thread::spawn(|| unsafe {
        unsafe extern "C" fn fiber_entry(user_data: *mut ()) {
            let message = unsafe { &*(user_data as *const &str) };
            println!(
                "[second fiber] Main fiber left us a message: \"{}\"",
                message
            );
            println!("[second fiber] Switching back to main fiber...");
            DefaultFiberApi::switch_to_fiber(SECOND_FIBER, MAIN_FIBER);
            println!("[second fiber] Back in the second fiber after main fiber resumed us!");
            println!("[second fiber] Fiber execution complete.");
            DefaultFiberApi::switch_to_fiber(SECOND_FIBER, MAIN_FIBER);
        }

        let main_fiber =
            DefaultFiberApi::convert_thread_to_fiber().expect("Failed to convert thread to fiber");
        MAIN_FIBER = main_fiber;

        let message = "Hello from the main fiber!";
        let user_data = &message as *const &str as *mut ();

        let stack = FiberStack::new(STACK_SIZE);
        let fiber = DefaultFiberApi::create_fiber(stack.as_pointer(), fiber_entry, user_data)
            .expect("Failed to create fiber");
        SECOND_FIBER = fiber;

        DefaultFiberApi::switch_to_fiber(MAIN_FIBER, SECOND_FIBER);

        println!("Back in the main fiber!");
        println!("Letting the fiber run its second half...");

        DefaultFiberApi::switch_to_fiber(MAIN_FIBER, SECOND_FIBER);

        println!("Fiber has finished execution!");
    })
    .join().expect("Thread panicked");
}
