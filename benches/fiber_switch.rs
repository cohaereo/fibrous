use criterion::{criterion_group, criterion_main, Criterion};

use fibrous::{
    sys::fcontext::FContextFiberApi, sys::ucontext::UContextFiberApi, FiberApi, FiberHandle,
    FiberStack,
};

struct PingPongData {
    main_handle: FiberHandle,
    worker_handle: FiberHandle,
    counter: usize,
}

extern "C" fn worker_entry_fn(data: *mut ()) {
    unsafe {
        let ctx = &mut *(data as *mut PingPongData);

        loop {
            let switch_fn: unsafe fn(FiberHandle, FiberHandle) = std::mem::transmute(ctx.counter);

            switch_fn(ctx.worker_handle, ctx.main_handle);
        }
    }
}

fn bench_implementation<Api: FiberApi>(c: &mut Criterion, name: &str) {
    c.bench_function(name, |b| {
        unsafe {
            let main_handle = Api::convert_thread_to_fiber().unwrap();
            let stack = FiberStack::new(1024 * 64); // 64kb stack

            let mut data = PingPongData {
                main_handle,
                worker_handle: FiberHandle::null(),
                counter: Api::switch_to_fiber as *const () as usize,
            };

            let worker_handle = Api::create_fiber(
                stack.as_pointer(),
                worker_entry_fn,
                &raw mut data as *mut (),
            )
            .unwrap();

            data.worker_handle = worker_handle;

            b.iter(|| {
                Api::switch_to_fiber(main_handle, worker_handle);

                // Prevent optimization
                std::hint::black_box(&mut data);
            });

            // Cleanup
            Api::destroy_fiber(worker_handle);
        }
    });
}

fn fiber_benchmark(c: &mut Criterion) {
    bench_implementation::<FContextFiberApi>(c, "fcontext_switch");
    bench_implementation::<UContextFiberApi>(c, "ucontext_switch");
}

criterion_group!(benches, fiber_benchmark);
criterion_main!(benches);
