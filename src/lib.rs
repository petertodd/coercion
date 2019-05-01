#![cfg_attr(feature = "maybe_uninit", feature(maybe_uninit))]

//! In-place conversions between types of the same size and alignment.
//!
//! # Examples
//!
//! ```
//! use coercion::{Coerce, As};
//!
//! let u8_slice: Box<[u8]> = vec![1,0,1,0].into_boxed_slice();
//!
//! // Safe because true as u8 == 1
//! let bool_slice: Box<[bool]> = unsafe { u8_slice.coerce() };
//!
//! assert_eq!(&bool_slice[..], &[true, false, true, false]);
//!
//! // Don't need unsafe, because any bool is a valid u8
//! let u8_slice: Box<[u8]> = bool_slice.as_();
//! assert_eq!(&u8_slice[..], &[1,0,1,0]);
//! ```
//!
//! Both `Coerce` and `As` are implemented for `str`:
//!
//! ```
//! use coercion::{Coerce, As};
//!
//! let boxed_str = Box::<str>::from("Hello World!");
//!
//! let boxed_bytes = boxed_str.as_();
//! assert_eq!(&boxed_bytes[..], b"Hello World!");
//!
//! let boxed_str: Box<str> = unsafe { boxed_bytes.coerce() };
//! assert_eq!(&*boxed_str, "Hello World!");
//! ```

mod coerce;
pub use self::coerce::Coerce;

mod as_;
pub use self::as_::As;

/// Implements `Coerce` for sized types.
#[macro_export]
macro_rules! unsafe_impl_coerce {
    ( $t:ty => $u:ty ) => {
        unsafe impl $crate::coerce::Coerce<$u> for $t {
            #[inline(always)]
            unsafe fn coerce(self) -> $u {
                ::core::mem::transmute(self)
            }

            #[inline(always)]
            fn coerce_ptr(this: *const Self) -> *const $u {
                this as *const $u
            }
        }
    };
    ( $t:ty | $u:ty ) => {
        $crate::unsafe_impl_coerce! { $t => $u }
        $crate::unsafe_impl_coerce! { $u => $t }
    };
    ( $t:ty | $u:ty | $( $u_remaining:ty )|+ ) => {
        $crate::unsafe_impl_coerce! {
            $t | $u
        }

        $(
            $crate::unsafe_impl_coerce! {
                $t | $u_remaining
            }
        )+

        $crate::unsafe_impl_coerce! {
            $u | $( $u_remaining )|+
        }
    };
    ( $( { $t:ty | $( $u:ty )|+ }; )+ ) => {
        $(
            $crate::unsafe_impl_coerce! {
                $t | $( $u )|+
            }
        )+
    };
}

/// Implements `As` for a sized types.
#[macro_export]
macro_rules! unsafe_impl_as {
    ( $src:ty => $( $dst:ty ),+ ) => {
        $(
            unsafe impl $crate::as_::As<$dst> for $src {}
        )+
    };

    ($src:ty = $dst:ty ) => {
        unsafe impl $crate::as_::As<$dst> for $src {}
        unsafe impl $crate::as_::As<$src> for $dst {}
    };

    ($src:ty = $dst:ty $(= $rest:ty )+ ) => {
        $crate::unsafe_impl_as!($src = $dst);

        $crate::unsafe_impl_as!($dst $( = $rest)+ );
    };

    ({ $src:ty => $( $dst:ty ),+ }) => {
        $crate::unsafe_impl_as!{ $src => $( $dst ),+ }
    };

    ({ $src:ty = $( $dst:ty )=+ }) => {
        $crate::unsafe_impl_as!{ $src = $( $dst )=+ }
    };

    ( $( $def:tt; )+ ) => {
        $(
            $crate::unsafe_impl_as! { $def }
        )+
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
