use core::mem::ManuallyDrop;

use crate::Coerce;

/// Types that can be reinterpreted as other types of the same size and alignment.
pub unsafe trait As<T: ?Sized> : Coerce<T> {
    /// Performs the conversion on an owned, sized, value.
    ///
    /// # Examples
    ///
    /// ```
    /// use coercion::As;
    ///
    /// let n = true.as_();
    /// assert_eq!(n, 1u8);
    /// ```
    fn as_(self) -> T
        where Self: Sized, T: Sized
    {
        unsafe {
            self.coerce()
        }
    }
}


unsafe impl<T, U> As<[U]> for [T]
where T: As<U>,
{}

unsafe impl<T: ?Sized, U: ?Sized> As<Box<U>> for Box<T>
where T: As<U>
{}

unsafe impl As<[u8]> for str
{}

unsafe impl<'a, T: ?Sized, U: ?Sized> As<&'a T> for &'a U
where T: As<U>,
{}

unsafe impl<'a, T: ?Sized, U: ?Sized> As<&'a mut T> for &'a mut U
where T: As<U>,
{}

unsafe impl<'a, T: ?Sized, U: ?Sized> As<*const T> for *const U
where T: As<U>,
{}

unsafe impl<'a, T: ?Sized, U: ?Sized> As<*mut T> for *mut U
where T: As<U>,
{}

unsafe impl<T: ?Sized, U: ?Sized> As<ManuallyDrop<U>> for ManuallyDrop<T>
where T: As<U>,
{}

mod impls {
    use crate::unsafe_impl_as;

    use core::num::*;
    use core::sync::atomic::*;

    unsafe_impl_as! {
        {u8 = i8 = AtomicU8 = AtomicI8};
        {u16 = i16 = AtomicU16 = AtomicI16};
        {u32 = i32 = AtomicU32 = AtomicI32};
        {u64 = i64 = AtomicU64 = AtomicI64};
        {u128 = i128 };
        {usize = isize = AtomicUsize = AtomicIsize};

        {bool => u8};
        {NonZeroU8 => u8};
        {NonZeroU16 => u16};
        {NonZeroU32 => u32};
        {NonZeroU64 => u64};
        {NonZeroU128 => u128};
        {NonZeroUsize=> usize};
        {NonZeroI8 => i8};
        {NonZeroI16 => i16};
        {NonZeroI32 => i32};
        {NonZeroI64 => i64};
        {NonZeroI128 => i128};
        {NonZeroIsize=> isize};
    }
}

#[cfg(test)]
mod tests {
    use super::As;

    #[test]
    fn boxed_str_to_u8() {
        let boxed_str: Box<str> = Box::from("Hello World!");
        let boxed_u8: Box<[u8]> = boxed_str.as_();
        assert_eq!(&boxed_u8[..], b"Hello World!");
    }
}
