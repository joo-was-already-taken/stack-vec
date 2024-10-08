use super::StackVec;

use std::mem::{self, ManuallyDrop, MaybeUninit};
use std::ptr;

pub struct IntoIter<T, const N: usize> {
    raw_iter: RawIter<T>,
    initial_len: usize,
    data: [MaybeUninit<T>; N],
}

impl<T, const N: usize> IntoIter<T, N> {
    pub fn len(&self) -> usize {
        self.raw_iter.len()
    }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        unsafe {
            for elem in &mut self.data[0..self.initial_len] {
                ptr::drop_in_place(elem.as_mut_ptr() as *mut T);
            }
        }
    }
}

impl<T, const N: usize> IntoIterator for StackVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let me = ManuallyDrop::new(self);
            let mut into_iter = IntoIter {
                raw_iter: RawIter {
                    begin: me.data.as_ptr() as *const T,
                    end: me.data.as_ptr().add(me.len) as *const T,
                },
                initial_len: me.len,
                data: MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init(),
            };
            ptr::copy_nonoverlapping(me.data.as_ptr(), into_iter.data.as_mut_ptr(), me.len);
            into_iter
        }
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw_iter.size_hint()
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw_iter.next_back()
    }
}

// pub struct Drain<'a, T: 'a, const N: usize> {
//     raw_iter: RawIter<T>,
//     _phantom: PhantomData<&'a mut StackVec<T, N>>,
// }
//
// impl<T, const N: usize> Iterator for Drain<'_, T, N> {
//     type Item = T;
//     
//     fn next(&mut self) -> Option<Self::Item> {
//         self.raw_iter.next()
//     }
//
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.raw_iter.size_hint()
//     }
// }
//
// impl<T, const N: usize> DoubleEndedIterator for Drain<'_, T, N> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.raw_iter.next_back()
//     }
// }
//
// impl<T, const N: usize> StackVec<T, N> {
//     pub fn drain(&mut self) -> Drain<T, N> {
//         let raw_iter = RawIter::new(&self);
//         self.len = 0;
//         Drain {
//             raw_iter,
//             _phantom: PhantomData,
//         }
//     }
// }

struct RawIter<T> {
    begin: *const T,
    end: *const T,
}

impl<T> RawIter<T> {
    // fn new(slice: &[T]) -> Self {
    //     let end = if mem::size_of::<T>() == 0 {
    //         (slice.as_ptr() as usize + slice.len()) as *const T
    //     } else {
    //         unsafe { slice.as_ptr().add(slice.len()) }
    //     };
    //
    //     Self {
    //         begin: slice.as_ptr(),
    //         end,
    //     }
    // }

    fn len(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            self.end as usize - self.begin as usize
        } else {
            (self.end as usize - self.begin as usize)  / mem::size_of::<T>()
        }
    }
}

impl<T> Iterator for RawIter<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            None
        } else {
            unsafe {
                let next = ptr::read(self.begin);
                self.begin = self.begin.add(1);
                Some(next)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.begin == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.sub(1);
                Some(ptr::read(self.end))
            }
        }
    }
}
