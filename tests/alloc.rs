use zeroizing_alloc::ZeroAlloc;

#[global_allocator]
static ALLOC: ZeroAlloc<std::alloc::System> = ZeroAlloc(std::alloc::System);

#[test]
fn can_alloc() {
    let allocation = core::hint::black_box(std::vec![1, 1, 1, 2, 2, 2]);
    drop(allocation); // Cannot check if zeroed post-drop without UB

    let mut allocation_2 = core::hint::black_box(Vec::<u8>::with_capacity(2));
    allocation_2.resize(2048, 0xFF);
    drop(allocation_2); // Cannot check if zeroed post-drop without UB
}
