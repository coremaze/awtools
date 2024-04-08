use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct DebugAllocator {
    used_bytes: AtomicUsize,
}

unsafe impl GlobalAlloc for DebugAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.used_bytes.fetch_add(layout.size(), Ordering::SeqCst);
        let used = self.used_bytes.load(Ordering::SeqCst);

        if used > 1_000_000_000 {
            eprintln!("Total allocation has grown to {used}. Shutting down.");
            panic!();
        }

        if layout.size() > 1_000_000 {
            eprintln!("Warning: Large allocation of {} bytes.", layout.size());
        }

        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.used_bytes.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout);
    }
}

#[global_allocator]
static GLOBAL: DebugAllocator = DebugAllocator {
    used_bytes: AtomicUsize::new(0),
};
