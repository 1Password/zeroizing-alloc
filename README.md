## About

This is a proof-of-concept crate for a [Global Allocator](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) in Rust that securely zeroizes all objects
upon deallocation, with a very low performance impact. It otherwise wraps the provided allocator and keeps its behavior.

### Example

To use this, you must define an allocator in your top-level binary or shared library. This looks like the following:
```rust
use zeroizing_alloc::ZeroAlloc;

#[global_allocator]
static ALLOC: ZeroAlloc<std::alloc::System> = ZeroAlloc(std::alloc::System);
```

### Contributions
We believe this crate to be feature-complete for its intended use cases. While PRs are always welcome, please keep in mind that the effort to verify the 
correctness and performance of changes made may not be worthwhile when weighed against the changeset itself.

### Research

On semi-recent Apple platforms (macOS 13+, iOS/tvOS 16.1+), the default allocator in `libSystem` [started zeroizing on free() by default.](https://mjtsai.com/blog/2022/09/20/zeroing-freed-memory/). 
This functionality is better optimized and more reliable than this wrapper, so it may be preferred. However, it is possible to [disable the behavior](https://github.com/apple-oss-distributions/libmalloc/blob/ac949e88b5b5fb90bf2e051c8a73754136ff1b43/private/malloc_private.h#L99)
in a few ways depending on your threat model.

## Credits

Made with ❤️ by the [1Password](https://1password.com/) data security team.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
