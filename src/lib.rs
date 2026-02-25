//! `slicewrap` - A macro for transparently wrapping slices in a type-safe manner.
//!
//! This crate provides a single macro for generating unit structs wrapping
//! (unsized) slices or strings with safe conversion functions.
//! While it is possible to write such structs, e.g. `pub struct StrWrap(str)`,
//! it is not possible to create instances of such types without using `unsafe`
//! pointer casts or transmutation.
//! The macro takes care of the unsafe code for generating the necessary
//! conversion as well as convenient trait implementations.
//! All generated functions for creating instances are private, so the user
//! is free to implement further (public) constructor functions with potential
//! additional invariant checks within the same module where the macro is
//! invoked.
//!
//! See the documentation of the [`slicewrap::wrap`] macro for details and
//! examples.

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
#[doc(hidden)]
pub mod __alloc {
    pub use alloc::boxed::Box;
    pub use alloc::rc::Rc;
    pub use alloc::sync::Arc;
}

/// A macro for generating the boilerpate code required for transparent newtype
/// unit struct wrappers around unsized slices (`[T]`) and `str`s.
///
/// For wrappers around [`str`], implementations for [`Display`](core::fmt::Display)
/// as well as direct comparisons with raw strings are also generated for
/// convenience.
///
/// # Examples
///
/// ```
/// slicewrap::wrap! {
///     /// A short string that is at most 8 bytes long.
///     #[derive(Debug, PartialEq)]
///     pub struct ShortStr(str);
/// }
///
/// // users may implement further constructors or inherent methods
/// impl ShortStr {
///     // external callers can only ever instantiate `ShortStr` from strings
///     // that are at most 8 bytes long
///     pub fn from_short_str(string: &str) -> Option<&Self> {
///         if string.len() <= 8 {
///             // the generated `from_ref` is private and only callable from
///             // within the same module as the struct declaration.
///             Some(Self::from_ref(string))
///         } else {
///             None
///         }
///     }
/// }
///
/// // `from_ref` is private
/// let strw = ShortStr::from_ref("Hello World");
/// assert_eq!(strw, "Hello World");
/// // `from_short_str` is public
/// let res = ShortStr::from_short_str("Hello World");
/// assert_eq!(res, None);
/// ```
///
/// The macro invocation can optionally generate safe zero-copy conversion
/// functions for any or all of the standard smart pointer types `Box`, `Rc` or
/// `Arc`:
///
/// ```
/// slicewrap::wrap!(
///     /// A tiny slice with at most 4 elements.
///     #[derive(Debug)]
///     pub struct TinySlice([u64]), from = [Box, Rc, Arc];
/// );
///
/// impl TinySlice {
///     // returns a tiny slice if `slice` is at most 4 elements long
///     pub fn new(slice: &[u64]) -> Option<&Self> {
///         if slice.len() <= 4 { Some(Self::from_ref(slice)) } else { None }
///     }
/// }
/// ```
///
/// # Note
///
/// It is currently not possible to wrap generic slices or slices of types with
/// lifetimes.
#[macro_export]
macro_rules! wrap {
    // The entry point for any slice wrapper type.
    ($(#[$attr:meta])* $vis:vis struct $name:ident([$type:ty]) $(, from = [$($from:ident),*])? $(;)?) => {
        $crate::wrap!(@inner $(#[$attr])* $vis struct $name ([$type]) $(, from = [$($from),*])?);
    };
    // The entry point for `str` slice wrappers (generates extra conversion &
    // comparison methods).
    ($(#[$attr:meta])* $vis:vis struct $name:ident(str) $(, from = [$($from:ident),*])? $(;)?) => {
        $crate::wrap!(@inner $(#[$attr])* $vis struct $name(str) $(, from = [$($from),*])?);

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }

        impl core::cmp::PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                &self.0 == other
            }
        }

        impl core::cmp::PartialOrd<str> for $name {
            fn partial_cmp(&self, other: &str) -> Option<core::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
    // internal: Generates base declarations and then any optional conversions.
    (@inner $(#[$attr:meta])* $vis:vis struct $name:ident ($type:ty) $(, from = [$($from:ident),*])? $(;)?) => {
        $crate::wrap!(@inner_base $(#[$attr])* $vis struct $name ($type));

        $($(
            $crate::wrap!(@inner_from $name $from $type);
        )*)?
    };
    // internal: Generates base declarations.
    (@inner_base $(#[$attr:meta])* $vis:vis struct $name:ident ($type:ty)) => {
        $(#[$attr])*
        #[repr(transparent)]
        $vis struct $name ($type);

        impl $name {
            #[allow(unused)]
            const fn from_ref(reference: &$type) -> &Self {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(reference) }
            }

            #[allow(unused)]
            fn from_ref_mut(reference: &mut $type) -> &mut Self {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(reference) }
            }

            const fn as_inner(&self) -> &$type {
                &self.0
            }

            fn as_inner_mut(&mut self) -> &mut $type {
                &mut self.0
            }
        }

        impl core::ops::Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl AsRef<$type> for $name {
            fn as_ref(&self) -> &$type {
                self.as_inner()
            }
        }

        impl AsMut<$type> for $name {
            fn as_mut(&mut self) -> &mut $type {
                self.as_inner_mut()
            }
        }
    };
    // Generates from/into functions for conversion of `Box` slices.
    (@inner_from $name:ident Box $type:ty) => {
        impl $name {
            const fn from_boxed(
                boxed: $crate::__alloc::Box<$type>
            ) -> $crate::__alloc::Box<Self>
            {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(boxed) }
            }

            #[allow(unused)]
            const fn into_boxed(
                self: $crate::__alloc::Box<Self>
            ) -> $crate::__alloc::Box<$type>
            {
               // SAFETY: the wrapper is a transparent newtype
               unsafe { core::mem::transmute(self) }
            }
        }

        impl From<&$name> for $crate::__alloc::Box<$name> {
            fn from(reference: &$name) -> $crate::__alloc::Box<$name> {
                let boxed: $crate::__alloc::Box<$type> = (&reference.0).into();
                $name::from_boxed(boxed)
            }
        }
    };
    // Generates from/into functions for conversion of `Rc` slices.
    (@inner_from $name:ident Rc $type:ty) => {
        impl $name {
            const fn from_rc(
                rc: $crate::__alloc::Rc<$type>
            ) -> $crate::__alloc::Rc<Self> {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(rc) }
            }

            #[allow(unused)]
            const fn into_rc(
                self: $crate::__alloc::Rc<Self>
            ) -> $crate::__alloc::Rc<$type> {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(self) }
            }
        }
    };
    // Generates from/into functions for conversion of `Arc` slices.
    (@inner_from $name:ident Arc $type:ty) => {
        impl $name {
            const fn from_arc(
                arc: $crate::__alloc::Arc<$type>
            ) -> $crate::__alloc::Arc<Self> {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(arc) }
            }

            #[allow(unused)]
            const fn into_arc(
                self: $crate::__alloc::Arc<Self>
            ) -> $crate::__alloc::Arc<$type> {
                // SAFETY: the wrapper is a transparent newtype
                unsafe { core::mem::transmute(self) }
            }
        }
    }
}

/// A macro for ergonomic matching on optional wrapped types created with
/// `slicewrap::wrap`.
///
/// # Examples
///
/// ```
/// slicewrap::wrap!(struct Str(str));
///
/// let opt: Option<&Str> = Some(Str::from_ref("foo"));
/// // since `opt` is an optional reference to a `Str` (uppercase) and the
/// // generic `Deref` implementation for `&T` simply derefs to `&T` again,
/// // calling `Option::as_deref` simply yields another `&Str` reference, which
/// // is not helpful here
/// let as_deref: Option<&Str> = opt.as_deref();
/// // using the `as_deref` macro we can ergonomically get the desired result
/// match slicewrap::as_deref!(opt) {
///     Some("foo") => println!("this is what we wanted"),
///     _ => unreachable!(),
/// }
/// ```
///
/// For retrieving a mutable reference, prepend `mut` to the macro argument:
///
/// ```
/// slicewrap::wrap!(struct Str(str));
///
/// let mut opt: Option<&mut Str> = None;
/// assert_eq!(slicewrap::as_deref!(mut opt), None);
/// ```
#[macro_export]
macro_rules! as_deref {
    ($wrap:expr) => {{
        match $wrap {
            Some(ref inner) => Some(inner.as_inner()),
            None => None,
        }
    }};
    (mut $wrap:expr) => {{
        match $wrap {
            Some(ref mut inner) => Some(inner.as_inner_mut()),
            None => None,
        }
    }};
}

#[cfg(test)]
mod tests {
    use std::{rc::Rc, sync::Arc};

    super::wrap!(
        /// Some documentation.
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Simple(str)
    );

    super::wrap!(
        #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        struct Heapable(str), from = [Box, Rc]
    );

    super::wrap!(pub struct SliceWrap([u8]), from = [Arc, Box, Rc]);

    impl Heapable {
        fn to_boxed(&self) -> Box<Self> {
            Self::from_boxed(self.0.into())
        }
    }

    #[test]
    fn simple() {
        let s = Simple::from_ref("simple");
        assert_eq!(s, "simple");

        let as_ref: &str = s.as_ref();
        assert_eq!(as_ref, "simple");

        let debug = format!("{:?}", s);
        assert_eq!(debug, "Simple(\"simple\")");

        let display = format!("{}", s);
        assert_eq!(display, "simple");

        let mut src = "mutable".to_string();
        let m = Simple::from_ref_mut(&mut src);
        m.as_mut().make_ascii_uppercase();
        assert_eq!(m, "MUTABLE");
    }

    #[test]
    fn heapable() {
        let not_on_heap = Heapable::from_ref("test");
        let boxed = not_on_heap.to_boxed();
        assert_eq!(boxed.as_ref(), "test");

        let rc: Rc<str> = Rc::from("heapable");
        let he = Heapable::from_rc(rc);
        assert_eq!(he.as_ref(), "heapable");
    }

    #[test]
    fn slicewrap() {
        let buf = &[0u8, 1, 2, 3];
        let bufw = SliceWrap::from_ref(buf);

        let boxed: Box<_> = bufw.as_ref().into();
        let boxed: Box<SliceWrap> = SliceWrap::from_boxed(boxed);
        assert_eq!(boxed.as_inner(), &[0, 1, 2, 3]);

        let rc: Rc<_> = bufw.as_ref().into();
        let rc: Rc<SliceWrap> = SliceWrap::from_rc(rc);
        assert_eq!(rc.as_inner(), &[0, 1, 2, 3]);

        let arc: Arc<_> = bufw.as_ref().into();
        let arc: Arc<SliceWrap> = SliceWrap::from_arc(arc);
        assert_eq!(arc.as_inner(), &[0, 1, 2, 3]);
    }

    #[test]
    fn deref() {
        let bufw = SliceWrap::from_ref(&[0, 1, 2, 3]);
        assert_eq!(&bufw[..2], &[0, 1]);
        assert_eq!(bufw.split_at(2), (&[0, 1][..], &[2, 3][..]));
    }

    #[test]
    fn as_deref() {
        let opt = Some(Simple::from_ref("foo"));
        assert_eq!(super::as_deref!(opt), Some("foo"));

        let mut string = String::from("foo");
        let mut opt = Some(Simple::from_ref_mut(string.as_mut_str()));
        if let Some(v) = super::as_deref!(mut opt) {
            v.make_ascii_uppercase();
        }

        assert_eq!(super::as_deref!(opt), Some("FOO"));
    }
}
