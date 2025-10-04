use core::alloc::{GlobalAlloc, Layout};
use std::{
    alloc::System,
    sync::{Mutex, OnceLock},
};
use zeroizing_alloc::ZeroAlloc;

#[global_allocator]
static ALLOC: ZeroAlloc<SpyAlloc<System>> = ZeroAlloc(SpyAlloc(System));

#[test]
fn can_alloc() {
    let allocation = core::hint::black_box(std::vec![1, 1, 1, 2, 2, 2]);
    drop(allocation); // Cannot check if zeroed post-drop without UB

    let mut allocation_2 = core::hint::black_box(Vec::<u8>::with_capacity(2));
    allocation_2.resize(2048, 0xFF);
    drop(allocation_2); // Cannot check if zeroed post-drop without UB
}

#[test]
fn freed_memory_is_zeroed() {
    SpyAlloc::<System>::clear_log();

    let allocation = core::hint::black_box(vec![1, 1, 1, 2, 2, 2]);
    drop(allocation);

    let freed = SpyAlloc::<System>::last_freed();
    assert_eq!(freed, [0; 32], "memory not zeroed: {freed:?}");

    let mut allocation_2 = core::hint::black_box(Vec::<u8>::with_capacity(2));
    allocation_2.resize(2048, 0xFF);
    drop(allocation_2);

    let freed = SpyAlloc::<System>::last_freed();
    assert_eq!(freed, [0; 32], "memory not zeroed: {freed:?}");
}

struct SpyAlloc<A: GlobalAlloc>(A);

impl<A: GlobalAlloc> SpyAlloc<A> {
    fn log() -> &'static Mutex<[u8; 32]> {
        static LOG: OnceLock<Mutex<[u8; 32]>> = OnceLock::new();
        LOG.get_or_init(|| Mutex::new([0; 32]))
    }

    fn last_freed() -> [u8; 32] {
        *Self::log().lock().unwrap()
    }

    fn clear_log() {
        let mut log = Self::log().lock().unwrap();
        log.fill(0);
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for SpyAlloc<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let slice = core::slice::from_raw_parts(ptr, layout.size());

        let mut log = Self::log().lock().unwrap();
        let len = slice.len().min(32);
        log[..len].copy_from_slice(&slice[..len]);
        if len < 32 {
            log[len..].fill(0);
        }

        self.0.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.0.alloc_zeroed(layout)
    }
}
