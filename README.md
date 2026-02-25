# `slicewrap` - Generating unsized newtype wrappers

This crate provides the `slicewrap::wrap!` macro for generating newtype wrapper
types for (unsized) strings and slices and generates the required
(partially unsafe) boilerplate code for actually creating such types as well as
using them ergonomically.

[![crates.io](https://img.shields.io/crates/v/slicewrap.svg)](https://crates.io/crates/slicewrap)

## Documentation

https://docs.rs/slicewrap

## Usage

To use this crate, add the following to your `Cargo.toml`

```toml
[dependencies]
slicewrap = "0.1.0"
```

## Examples

While Rust allows you create newtype wrappers for unsized strings and slices
like this:

```rust
/// A string that is guaranteed to be uppercase only.
#[repr(transparent)]
struct UppercaseStr(str);
```

There is as of the time of writing (Rust `1.68.0`) no way to actually create
such a type (or rather a reference to one) with safe code and requires pointer
casts.
Using the `slicewrap::wrap!` macro generates this type for you as well as all
the boilerplate for creating and working with it:

```rust
slicewrap::wrap! {
    /// A string that is guaranteed to be uppercase only.
    ///
    /// NOTE: `UppercaseStr` will be `#[repr(transparent)]` automatically.
    #[derive(Debug)]
    pub struct UppercaseStr(str);
}
```

Aside from generating useful trait implementations for the generated type, such
as `Display`, `AsRef<str>`, `PartialEq<str>` and others, this will generate two
constructor functions, that will be **private to the module where this macro is
invoked**.
These will be `UppercaseStr::from_ref` and `UppercaseStr::from_ref_mut`.
Further constructor functions can be written **manually** to enforce additional
type invariants, such as this:

```rust
impl UppercaseStr {
    pub fn from_str(string: &str) -> Option<&Self> {
        if string.chars().all(|c| c.is_uppercase()) {
            // use the private `from_ref` here, which can only be
            // called in the module the macro was invoked in and
            // its sub modules.
            Some(Self::from_ref(string))
        } else {
            None
        }
    }
}
```

## License

This project is licensed under either of

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT)
  ([LICENSE-MIT](LICENSE-MIT))

at your option.
