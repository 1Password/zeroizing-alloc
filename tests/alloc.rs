use core::alloc::{GlobalAlloc, Layout};
use quickcheck_macros::quickcheck;
use std::{alloc::System, cmp::min, sync::Mutex};
use zeroizing_alloc::ZeroAlloc;

const CAPACITY: usize = 2048;

#[global_allocator]
static ALLOC: ZeroAlloc<SpyAlloc<System, CAPACITY>> = ZeroAlloc(SpyAlloc(
    System,
    Mutex::new(AllocInfo {
        alloc_count: 0,
        zeroed: [false; CAPACITY],
    }),
));

#[test]
fn freed_memory_is_zeroed() {
    let allocation = core::hint::black_box(vec![1, 1, 1, 2, 2, 2]);
    drop(allocation);

    let mut allocation_2 = core::hint::black_box(Vec::<u8>::with_capacity(2));
    allocation_2.resize(2048, 0xFF);
    drop(allocation_2);

    assert!(&ALLOC.0.verify_allocs_zeroed());
}

#[quickcheck]
fn prop_allocations_are_zeroed(input: Vec<u32>) -> bool {
    // skip empty vectors since they dont allocate
    if input.is_empty() {
        return true;
    }
    drop(input);
    ALLOC.0.verify_allocs_zeroed()
}

#[derive(Clone, Copy)]
struct AllocInfo {
    alloc_count: usize,
    zeroed: [bool; CAPACITY],
}

struct SpyAlloc<A: GlobalAlloc, const CAPACITY: usize>(A, Mutex<AllocInfo>);

impl<A: GlobalAlloc, const CAPACITY: usize> SpyAlloc<A, CAPACITY> {
    fn verify_allocs_zeroed(&self) -> bool {
        let info = self.1.lock().unwrap();
        let allocs_to_check = min(CAPACITY, info.alloc_count);
        info.zeroed[..allocs_to_check].iter().all(|&b| b)
    }
}

unsafe impl<A: GlobalAlloc, const CAPACITY: usize> GlobalAlloc for SpyAlloc<A, CAPACITY> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let slice = core::slice::from_raw_parts(ptr, layout.size());

        let mut alloc_info = self.1.lock().unwrap();

        if slice.iter().all(|i| *i == 0) {
            let alloc_index = alloc_info.alloc_count % CAPACITY;
            alloc_info.zeroed[alloc_index] = true;
        }
        alloc_info.alloc_count += 1;

        self.0.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.0.alloc_zeroed(layout)
    }
}
