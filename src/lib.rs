//! A vector-like data structure with fixed capacity and residing on the stack.
//!
//! # Example
//! ```
//! # use stack_vec::*;
//! let mut vec = StackVec::<_, 96>::new();
//! vec.push(1);
//! vec.push(2);
//!
//! assert_eq!(vec, stack_vec![1, 2]);
//! assert_eq!(vec.len(), 2);
//! assert_eq!(vec.as_slice(), &[1, 2]);
//! ```

// uncomment for linting, comment before committing (backward compatibility)
// #![deny(unsafe_op_in_unsafe_fn)]

mod iter;
pub use iter::IntoIter;

mod macros;

#[cfg(test)]
mod tests;

use std::iter::FromIterator;
use std::mem::{self, MaybeUninit};
use std::ops;
use std::ptr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotEnoughSpaceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertError {
    IndexOutOfRange,
    NotEnoughSpace,
}

/// A vector-like data structure with fixed capacity and residing on the stack.
///
/// # Example
/// ```
/// # use stack_vec::*;
/// let mut vec = StackVec::<_, 96>::new();
/// vec.push(1);
/// vec.push(2);
///
/// assert_eq!(vec, stack_vec![1, 2]);
/// assert_eq!(vec.len(), 2);
/// assert_eq!(vec.as_slice(), &[1, 2]);
/// ```
#[derive(Debug)]
pub struct StackVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        unsafe {
            self.drop_range(0..self.len);
        }
    }
}

unsafe impl<T: Send, const N: usize> Send for StackVec<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StackVec<T, N> {}

impl<T, const N: usize> StackVec<T, N> {
    /// Length of an underlying array.
    pub const CAPACITY: usize = N;

    // #[rustversion::since(1.59)] // `MaybeUninit::assume_init` became const
    #[inline]
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    // #[rustversion::before(1.59)]
    // #[inline]
    // pub fn new() -> Self {
    //     Self {
    //         data: unsafe { mem::MaybeUninit::uninit().assume_init() },
    //         len: 0,
    //     }
    // }

    /// Constructs a new `StackVec<T, N>`.
    /// Returns `None` if provided array is longer than `N`.
    pub fn from_array<const M: usize>(arr: [T; M]) -> Option<Self> {
        if M > Self::CAPACITY {
            None
        } else {
            unsafe {
                let mut vec = Self::new();
                ptr::copy_nonoverlapping(arr.as_ptr(), vec.as_mut_ptr(), M);
                vec.set_len(M);
                Some(vec)
            }
        }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as _
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as _
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// Just a setter.
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= Self::CAPACITY);
        self.len = new_len;
    }

    /// Pushes a value after the last element, panics if there is not space available.
    /// See [`try_push`](StackVec::try_push) or [`push_unchecked`](StackVec::push_unchecked) for
    /// related methods.
    pub fn push(&mut self, value: T) {
        #[cold]
        #[track_caller]
        fn assert_failed(cap: usize) -> ! {
            panic!("push failed: not enough space in StackVec (capacity is {})", cap);
        }

        if self.len < Self::CAPACITY {
            unsafe { self.push_unchecked(value); }
        } else {
            assert_failed(Self::CAPACITY);
        }
    }

    /// Pushes a value after the last element returning a `Result`.
    /// See also [`push_unchecked`](StackVec::push_unchecked).
    pub fn try_push(&mut self, value: T) -> Result<(), NotEnoughSpaceError> {
        if self.len < Self::CAPACITY {
            unsafe { self.push_unchecked(value); }
            Ok(())
        } else {
            cold();
            Err(NotEnoughSpaceError)
        }
    }

    /// Pushes a value after the last element without any checks.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        unsafe {
            ptr::write(self.as_mut_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { self.drop_range(0..self.len); }
        self.len = 0;
    }

    /// Inserts a value at specified index by pushing elements from `idx` by one.
    /// Panics on invalid index.
    /// See also [`try_insert`](StackVec::try_insert) and [`insert_unchecked`](StackVec::insert_unchecked).
    pub fn insert(&mut self, idx: usize, value: T) {
        #[cold]
        #[track_caller]
        fn assert_idx_failed(idx: usize, len: usize) -> ! {
            panic!("insertion index (is {}) should be <= len (is {})", idx, len);
        }

        #[cold]
        #[track_caller]
        fn assert_len_failed(cap: usize) -> ! {
            panic!("insertion failed: not enough space in StackVec (capacity is {})", cap)
        }

        if idx > self.len {
            assert_idx_failed(idx, self.len);
        }
        if self.len >= Self::CAPACITY {
            assert_len_failed(Self::CAPACITY);
        }

        unsafe { self.insert_unchecked(idx, value); }
    }

    /// Inserts a value at specified index by pushing elements from `idx` by one.
    /// See also [`insert_unchecked`](StackVec::insert_unchecked).
    pub fn try_insert(&mut self, idx: usize, value: T) -> Result<(), InsertError> {
        if idx > self.len {
            cold();
            return Err(InsertError::IndexOutOfRange);
        }
        if self.len >= Self::CAPACITY {
            cold();
            return Err(InsertError::NotEnoughSpace);
        }

        unsafe { self.insert_unchecked(idx, value); }
        Ok(())
    }

    /// Inserts a value at specified index by pushing elements from `idx` by one without performing
    /// any checks.
    pub unsafe fn insert_unchecked(&mut self, idx: usize, value: T) {
        unsafe {
            let insert_ptr = self.as_mut_ptr().add(idx);
            ptr::copy(insert_ptr, insert_ptr.add(1), self.len - idx);
            ptr::write(insert_ptr, value);
        }
        self.len += 1;
    }

    /// Pops the last element from a [`StackVec`].
    /// If exists returns it in `Some`, otherwise `None`.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.len -= 1;
                Some(ptr::read(self.as_ptr().add(self.len)))
            }
        }
    }

    /// Removes an element specified by `idx`.
    /// Panics if `idx >= self.len`.
    /// See also [`try_remove`](StackVec::try_remove) and [`remove_unchecked`](StackVec::remove_unchecked).
    pub fn remove(&mut self, idx: usize) -> T {
        #[cold]
        #[track_caller]
        fn assert_failed(idx: usize, len: usize) -> ! {
            panic!("removal index (is {}) should be < len (is {})", idx, len);
        }

        if idx >= self.len {
            assert_failed(idx, self.len);
        }

        unsafe { self.remove_unchecked(idx) }
    }

    /// Removes an element specified by `idx`.
    /// Returns `None` if `idx` is out of range.
    /// See also [`remove_unchecked`](StackVec::remove_unchecked).
    pub fn try_remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.len {
            cold();
            None
        } else {
            unsafe { Some(self.remove_unchecked(idx)) }
        }
    }

    /// Removes an element specified by `idx` without any checks.
    pub unsafe fn remove_unchecked(&mut self, idx: usize) -> T {
        unsafe {
            self.len -= 1;
            let remove_ptr = self.as_mut_ptr().add(idx);
            let val = ptr::read(remove_ptr);
            ptr::copy(remove_ptr.add(1), remove_ptr, self.len - idx);
            val
        }
    }

    /// Truncates a [`StackVec`] to specified length.
    /// Does nothing if `new_len` is greater than current length.
    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        let old_len = self.len;
        unsafe { self.drop_range(new_len..old_len); }
        self.len = old_len.min(new_len);
    }

    unsafe fn drop_range(&mut self, range: std::ops::Range<usize>) {
        if range.start < range.end {
            unsafe {
                for elem in &mut self.data[range] {
                    ptr::drop_in_place(elem.as_mut_ptr() as *mut T);
                }
            }
        }
    }
}

impl<T: Copy, const N: usize> StackVec<T, N> {
    /// Creates a [`StackVec`] of a given size by copying provided value.
    /// Returns `None` if `len` is greater than [`StackVec::CAPACITY`].
    pub fn from_value(val: T, len: usize) -> Option<Self> {
        if len > Self::CAPACITY {
            None
        } else {
            unsafe {
                let mut vec = Self::new();
                let mut ptr = vec.as_mut_ptr();
                for _ in 0..len {
                    ptr::write(ptr, val);
                    ptr = ptr.add(1);
                }
                vec.set_len(len);
                Some(vec)
            }
        }
    }

    /// Resizes a [`StackVec`] to specified length.
    /// If `new_len` is greater than the current length - extends the [`StackVec`] with `val`.
    /// Panics if `new_len` is greater than [`StackVec::CAPACITY`].
    pub fn resize(&mut self, new_len: usize, val: T) {
        if new_len > self.len {
            self.extend_with(new_len - self.len, val);
        } else {
            self.truncate(new_len);
        }
    }

    /// Extends a [`StackVec`] by copying `val` `n` times.
    /// Panics if new length (old length + `n`) is greater than [`StackVec::CAPACITY`].
    pub fn extend_with(&mut self, n: usize, val: T) {
        #[cold]
        #[track_caller]
        fn assert_failed(cap: usize, req_cap: usize) -> ! {
            panic!("extend failed: capacity too low (is {}, required {})", cap, req_cap);
        }

        let new_len = self.len + n;
        if new_len > Self::CAPACITY {
            assert_failed(Self::CAPACITY, new_len);
        }

        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len);
            for _ in 0..n {
                ptr::write(ptr, val);
                ptr = ptr.add(1);
            }
        }
        self.len = new_len;
    }
}

impl<T: PartialEq, const N: usize> PartialEq for StackVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
        && self.iter()
            .zip(other.iter())
            .all(|(a, b)| a == b)
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ops::Deref for StackVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.as_ptr() as _, self.len)
        }
    }
}

impl<T, const N: usize> ops::DerefMut for StackVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            std::slice::from_raw_parts_mut(self.as_mut_ptr() as _, self.len)
        }
    }
}

impl<T, const N: usize> Extend<T> for StackVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        #[cold]
        #[track_caller]
        fn assert_failed() -> ! {
            panic!("Cannot extend `StackVec` with an iterator longer than the available space");
        }

        let mut iter = iter.into_iter();
        while let Some(elem) = iter.next() {
            let len = self.len();
            if len == Self::CAPACITY {
                assert_failed();
            }
            unsafe {
                ptr::write(self.as_mut_ptr().add(len), elem);
                self.len += 1;
            }
        }
    }
}

impl<T, const N: usize> AsMut<[T]> for StackVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> From<[T; N]> for StackVec<T, N> {
    #[inline]
    fn from(arr: [T; N]) -> Self {
        Self {
            data: unsafe { mem::transmute_copy(&arr) },
            len: N,
        }
    }
}

impl<T, const N: usize> FromIterator<T> for StackVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = StackVec::new();
        vec.extend(iter);
        vec
    }
}

// impl<T, const N: usize, const M: usize> TryFrom<[T; M]> for StackVec<T, N> {
//     type Error = NotEnoughSpaceError;
//
//     fn try_from(arr: [T; M]) -> Result<Self, Self::Error> {
//         Self::from_array(arr).ok_or(NotEnoughSpaceError)
//     }
// }

#[cold]
fn cold() {}
