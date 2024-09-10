// uncomment for linting, comment before committing (backward compatibility)
// #![deny(unsafe_op_in_unsafe_fn)]

mod iter;
pub use iter::IntoIter;

mod macros;

#[cfg(test)]
mod tests;

use std::mem;
use std::ops;
use std::ptr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotEnoughSpaceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertError {
    IndexOutOfRange,
    NotEnoughSpace,
}

#[derive(Debug, Clone)]
pub struct StackVec<T, const N: usize> {
    data: [T; N],
    len: usize,
}

unsafe impl<T: Send, const N: usize> Send for StackVec<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for StackVec<T, N> {}

impl<T, const N: usize> StackVec<T, N> {
    pub const CAPACITY: usize = N;

    #[rustversion::since(1.59)] // `MaybeUninit::assume_init` became const
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: unsafe { mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    #[rustversion::before(1.59)]
    #[inline]
    pub fn new() -> Self {
        Self {
            data: unsafe { mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

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
    pub const fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= Self::CAPACITY);
        self.len = new_len;
    }

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

    pub fn try_push(&mut self, value: T) -> Result<(), NotEnoughSpaceError> {
        if self.len < Self::CAPACITY {
            unsafe { self.push_unchecked(value); }
            Ok(())
        } else {
            cold();
            Err(NotEnoughSpaceError)
        }
    }

    pub unsafe fn push_unchecked(&mut self, value: T) {
        unsafe {
            ptr::write(self.as_mut_ptr().add(self.len), value);
        }
        self.len += 1;
    }

    #[inline]
    pub fn clear(&mut self) {
        // let elems: *mut [T] = self.as_mut_slice();
        self.len = 0;
    }

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

    pub unsafe fn insert_unchecked(&mut self, idx: usize, value: T) {
        unsafe {
            let insert_ptr = self.as_mut_ptr().add(idx);
            ptr::copy(insert_ptr, insert_ptr.add(1), self.len - idx);
            ptr::write(insert_ptr, value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(unsafe {
                ptr::read(self.as_ptr().add(self.len))
            })
        }
    }

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

    pub unsafe fn remove_unchecked(&mut self, idx: usize) -> T {
        unsafe {
            self.len -= 1;
            let remove_ptr = self.as_mut_ptr().add(idx);
            let val = ptr::read(remove_ptr);
            ptr::copy(remove_ptr.add(1), remove_ptr, self.len - idx);
            val
        }
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        self.len = self.len.min(new_len);
    }
}

impl<T: Copy, const N: usize> StackVec<T, N> {
    pub fn from_elem(elem: T, len: usize) -> Option<Self> {
        if len > Self::CAPACITY {
            None
        } else {
            unsafe {
                let mut vec = Self::new();
                let mut ptr = vec.as_mut_ptr();
                for _ in 0..len {
                    ptr::write(ptr, elem);
                    ptr = ptr.add(1);
                }
                vec.set_len(len);
                Some(vec)
            }
        }
    }

    pub fn resize(&mut self, new_len: usize, val: T) {
        if new_len > self.len {
            self.extend_with(new_len - self.len, val);
        } else {
            self.truncate(new_len);
        }
    }

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
                ptr::write(ptr, val.clone());
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
        &self.data[0..self.len]
    }
}

impl<T, const N: usize> ops::DerefMut for StackVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data[0..self.len]
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
            data: arr,
            len: N,
        }
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
