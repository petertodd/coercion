use core::alloc::Layout;
use core::mem::{self, ManuallyDrop};

#[cfg(feature = "maybe_uninit")]
use core::mem::MaybeUninit;

macro_rules! assert_layout_eq {
    ($t:ty, $u:ty) => {
        {
            let layout_t = Layout::new::<$t>();
            let layout_u = Layout::new::<$u>();
            assert_eq!(layout_t, layout_u);
        }
    }
}

macro_rules! assert_ptr_layout_eq {
    ($t:ty, $u:ty) => {
        {
            let layout_ptr_t = Layout::new::<*const $t>();
            let layout_ptr_u = Layout::new::<*const $u>();
            assert_eq!(layout_ptr_t, layout_ptr_u);
        }
    }
}

/// Unsafe, unchecked, coercion between types of the same size and alignment.
pub unsafe trait Coerce<T: ?Sized> {
    /// Performs the coercion on an owned value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::num::NonZeroU64;
    /// use coercion::Coerce;
    ///
    /// // Safe because 1 is non-zero.
    /// let nonzero: NonZeroU64 = unsafe { 1u64.coerce() };
    /// assert_eq!(nonzero.get(), 1);
    /// ```
    unsafe fn coerce(self) -> T
        where Self: Sized, T: Sized
    {
        assert_layout_eq!(Self, T);
        assert_ptr_layout_eq!(Self, T);

        let this = ManuallyDrop::new(self);
        let r = mem::transmute_copy(&this);
        r
    }

    /// Perform the conversion on a `const` pointer.
    fn coerce_ptr(this: *const Self) -> *const T {
        assert_ptr_layout_eq!(Self, T);

        let r = unsafe { mem::transmute_copy(&this) };
        r
    }

    /// Perform the conversion on a `mut` pointer.
    fn coerce_mut_ptr(this: *mut Self) -> *mut T {
        assert_ptr_layout_eq!(Self, T);

        Self::coerce_ptr(this) as *mut T
    }
}

#[repr(C)]
struct FatPtr {
    ptr: usize,
    len: usize,
}

unsafe impl<T, U> Coerce<[U]> for [T]
where T: Coerce<U>,
{
    fn coerce_ptr(this: *const Self) -> *const [U] {
        assert_layout_eq!(T,U);

        unsafe {
            let repr: FatPtr = mem::transmute(this);
            mem::transmute(repr)
        }
    }
}


unsafe impl<T: ?Sized, U: ?Sized> Coerce<Box<U>> for Box<T>
where T: Coerce<U>
{
    unsafe fn coerce(self) -> Box<U> {
        assert_ptr_layout_eq!(T, U);
        assert_layout_eq!(Box<T>,Box<U>);

        let t_ptr: *mut T = Box::into_raw(self);
        let u_ptr: *mut U = T::coerce_mut_ptr(t_ptr);

        Box::from_raw(u_ptr)
    }

    fn coerce_ptr(this: *const Self) -> *const Box<U> {
        this as *const Box<U>
    }
}

unsafe impl Coerce<str> for [u8] {
    #[inline(always)]
    fn coerce_ptr(this: *const Self) -> *const str {
        unsafe {
            mem::transmute(this)
        }
    }
}

unsafe impl Coerce<[u8]> for str {
    #[inline(always)]
    fn coerce_ptr(this: *const Self) -> *const [u8] {
        unsafe {
            mem::transmute(this)
        }
    }
}

#[cfg(feature = "maybe_uninit")]
unsafe impl<T> Coerce<T> for MaybeUninit<T> {
    fn coerce_ptr(this: *const Self) -> *const T {
        this as *const T
    }
}

#[cfg(feature = "maybe_uninit")]
unsafe impl<T> Coerce<MaybeUninit<T>> for T {
    fn coerce_ptr(this: *const Self) -> *const MaybeUninit<T> {
        this as *const _
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized> Coerce<&'a T> for &'a U
where T: Coerce<U>,
{
    fn coerce_ptr(this: *const Self) -> *const &'a T {
        this as *const &'a T
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized> Coerce<&'a mut T> for &'a mut U
where T: Coerce<U>,
{
    fn coerce_ptr(this: *const Self) -> *const &'a mut T {
        this as *const &'a mut T
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized> Coerce<*const T> for *const U
where T: Coerce<U>,
{
    fn coerce_ptr(this: *const Self) -> *const *const T {
        this as *const *const T
    }
}

unsafe impl<'a, T: ?Sized, U: ?Sized> Coerce<*mut T> for *mut U
where T: Coerce<U>,
{
    fn coerce_ptr(this: *const Self) -> *const *mut T {
        this as *const *mut T
    }
}

unsafe impl<T: ?Sized, U: ?Sized> Coerce<ManuallyDrop<U>> for ManuallyDrop<T>
where T: Coerce<U>,
{}

mod impls {
    use crate::unsafe_impl_coerce;

    use core::num::*;
    use core::sync::atomic::*;

    // primitive impls
    unsafe_impl_coerce! {
        { u8  | i8  | NonZeroU8  | NonZeroI8  | AtomicU8  | AtomicI8 | bool | AtomicBool };
        { u16 | i16 | NonZeroU16 | NonZeroI16 | AtomicU16 | AtomicI16 };
        { u32 | i32 | NonZeroU32 | NonZeroI32 | AtomicU32 | AtomicI32 };
        { u64 | i64 | NonZeroU64 | NonZeroI64 | AtomicU64 | AtomicI64 };
        { u128 | i128 | NonZeroU128 | NonZeroI128 };
        { usize | isize | NonZeroUsize | NonZeroIsize | AtomicUsize | AtomicIsize };
    }
}

#[cfg(test)]
mod tests {
    use crate::Coerce;

    use std::num::NonZeroU64;

    #[test]
    fn box_coercion() {
        let boxed_u64: Box<u64> = Box::new(1u64);

        let boxed_nonzero_u64: Box<NonZeroU64> = unsafe { boxed_u64.coerce() };

        assert_eq!(*boxed_nonzero_u64, NonZeroU64::new(1).unwrap());
    }

    #[test]
    fn boxed_slice_coercion() {
        let boxed_u64 = vec![1u64; 100].into_boxed_slice();

        let boxed_nonzero: Box<[NonZeroU64]> = unsafe { boxed_u64.coerce() };

        assert_eq!(boxed_nonzero, vec![NonZeroU64::new(1).unwrap(); 100].into_boxed_slice());
    }

    #[cfg(feature = "maybe_uninit")]
    #[test]
    fn maybe_uninit() {
        use core::mem::MaybeUninit;

        let mut uninit: Box<MaybeUninit<NonZeroU64>> = Box::new(MaybeUninit::uninit());

        uninit.write(NonZeroU64::new(123456).unwrap());

        unsafe {
            let init: Box<NonZeroU64> = uninit.coerce();

            assert_eq!(init.get(), 123456);
        }
    }

    #[cfg(feature = "maybe_uninit")]
    #[test]
    fn maybe_uninit_slice() {
        use core::mem::MaybeUninit;

        let mut uninit: Box<[MaybeUninit<bool>]> = vec![MaybeUninit::<bool>::uninit(); 100].into_boxed_slice();

        for b in &mut uninit[..] {
            b.write(true);
        }

        unsafe {
            let init: Box<[bool]> = uninit.coerce();

            assert_eq!(&init[..], &vec![true; 100][..]);
        }
    }
}
