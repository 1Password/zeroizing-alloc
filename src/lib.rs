#![no_std]

//! An example crate showing how to safely and performantly zero out all heap allocations in a process.
//!
//!
//! This crates makes the following changes from common zeroizing alloc implementations:
//!
//! - Introduce a faster zeroization implementation (original kept behind feature "reference_impl" for perf testing)
//! - Fix a potential casting bug
//! - Remove unit tests: although passing locally, they trigger UAF and UB, leading to inconsistency, which we don't want.
//!     - Used `MIRIFLAGS="-Zmiri-ignore-leaks" cargo +nightly miri test -p op-alloc`
//!
//! <https://rust.godbolt.org> was a tool used to partially verify that zeroization will NOT be optimized out at `-Copt-level=3`

use core::alloc::{GlobalAlloc, Layout};

/// Allocator wrapper that zeros on free
pub struct ZeroAlloc<Alloc: GlobalAlloc>(pub Alloc);

// Reference implementation. Performance-wise, this is the same as using the `zeroize` crate,
// because it uses the same logic:
//
// ```rust
// unsafe fn zero(ptr: *mut u8, size: usize) {
//     use zeroize::Zeroize;
//     core::slice::from_raw_parts_mut(ptr, size).zeroize();
// }
// ```
//
// SAFETY: exactly one callsite (below), always passes the correct size
#[cfg(feature = "reference_impl")]
#[inline]
unsafe fn zero(ptr: *mut u8, len: usize) {
    for i in 0..len {
        core::ptr::write_volatile(ptr.add(i), 0);
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[cfg(not(feature = "reference_impl"))]
unsafe fn clear_bytes(ptr: *mut u8, len: usize) {
    // We expect this to optimize into a `memset` for performance. Due to the use of raw pointers, the compiler
    // doesn't know that the slice we are wiping is about to be destroyed anyway.
    //
    // SAFETY: The caller must only pass a valid allocated object.
    ptr.write_bytes(0x0, len);
}

// This is meant to avoid compiler optimizations while still retaining performance.
//
// By storing a function to a performant `memset(0, dest)` call, we can performantly zero out bytes
// without the compiler realizing the values being cleared aren't going to be read from again since it does
// not know either the source of the bytes or the source of our clearing function.
//
// - By loading this function pointer volatilely, we ensure the compiler does not optimize thinking about the
// source of the function pointer.
// - `#[used]` presents an extra optimization barrier since it forces the compiler to keep it around (won't take part in codegen optimization)
// until it reaches the linker. Even if the linker removes it though, its still fine because that can't optimize code that depends on it.
#[cfg(not(feature = "reference_impl"))]
#[used]
static WIPER: unsafe fn(*mut u8, usize) = clear_bytes;

// SAFETY: exactly one callsite (below), always passes the correct size
#[cfg(not(feature = "reference_impl"))]
#[inline]
unsafe fn zero(ptr: *mut u8, len: usize) {
    // The compiler may not predict anything about the clearing function we load due to the `read_volatile`, so
    // it must always load it from the static's address instead of directly calling the `clear_bytes` function (which
    // might allow optimizing away clearing).
    //
    // SAFETY: This static is always initialized to the correct value.
    let wipe = unsafe { core::ptr::addr_of!(WIPER).read_volatile() };
    wipe(ptr, len);
}

// SAFETY: wrapper for system allocator, zeroizes on free but otherwise re-uses system logic
unsafe impl<T> GlobalAlloc for ZeroAlloc<T>
where
    T: GlobalAlloc,
{
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        zero(ptr, layout.size());
        self.0.dealloc(ptr, layout);
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.0.alloc_zeroed(layout)
    }
}
