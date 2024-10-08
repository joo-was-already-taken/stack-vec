use super::*;

#[test]
fn new() {
    let vec = StackVec::<i32, 7>::new();
    assert_eq!(vec.len(), 0);

    let vec = StackVec::<i32, 0>::new();
    assert_eq!(vec.len(), 0);

    assert_eq!(
        StackVec::<i32, 4>::new(),
        StackVec::<i32, 4>::from_array([]).unwrap(),
    );
}

#[test]
fn push() {
    let mut vec = StackVec::<_, 4>::new();
    vec.push(0);
    assert_eq!(vec, stack_vec![0]);
    vec.push(1);
    assert_eq!(vec, stack_vec![0, 1]);
    let push_res = vec.try_push(2);
    assert_eq!(vec, stack_vec![0, 1, 2]);
    assert_eq!(push_res, Ok(()));
    vec.push(3);
    assert_eq!(vec, stack_vec![0, 1, 2, 3]);
    assert_eq!(vec.try_push(4), Err(NotEnoughSpaceError));
}

#[test]
fn push_zst() {
    #[derive(Clone, Copy)]
    struct Zst;

    let mut vec = stack_vec![Zst; 11; cap = 11];
    assert_eq!(vec.len(), 11);
    assert_eq!(vec.try_push(Zst), Err(NotEnoughSpaceError));
    assert_eq!(vec.len(), 11);
}

#[test]
fn insert() {
    let mut vec = stack_vec![1, 4, 5; cap = 7];
    vec.insert(1, 3);
    assert_eq!(vec, stack_vec![1, 3, 4, 5]);
    vec.insert(1, 2);
    assert_eq!(vec, stack_vec![1, 2, 3, 4, 5]);
    vec.insert(0, 0);
    assert_eq!(vec, stack_vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(vec.try_insert(7, 69), Err(InsertError::IndexOutOfRange));
    vec.insert(6, 6);
    assert_eq!(vec, stack_vec![0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(vec.try_insert(4, 69), Err(InsertError::NotEnoughSpace));
    assert_eq!(vec.try_insert(11, 69), Err(InsertError::IndexOutOfRange));
}

#[test]
fn pop() {
    let mut vec = stack_vec![1, 2, 3; cap = 3];
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec, stack_vec![1, 2]);
    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec, stack_vec![1]);
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec, stack_vec![]);
    assert_eq!(vec.pop(), None);
}

#[test]
fn remove() {
    let mut vec = stack_vec![1, 2, 3, 4; cap = 4];
    vec.remove(1);
    assert_eq!(vec, stack_vec![1, 3, 4]);
    assert_eq!(vec.try_remove(3), None);
    vec.remove(2);
    assert_eq!(vec, stack_vec![1, 3]);
    vec.remove(0);
    assert_eq!(vec, stack_vec![3]);
    vec.remove(0);
    assert_eq!(vec, stack_vec![]);
    assert_eq!(vec.try_remove(1), None);
    assert_eq!(vec.try_remove(0), None);
}

#[test]
fn truncate() {
    let mut vec = stack_vec![0, 1, 2, 3; cap = 4];
    vec.truncate(2);
    assert_eq!(vec, stack_vec![0, 1]);
    vec.truncate(100000);
    assert_eq!(vec, stack_vec![0, 1]);
    vec.truncate(0);
    assert_eq!(vec, stack_vec![]);
}

#[test]
fn resize() {
    let mut vec = stack_vec![0, 1, 2, 3, 4; cap = 10];
    vec.resize(9, 69);
    assert_eq!(vec, stack_vec![0, 1, 2, 3, 4, 69, 69, 69, 69]);
    vec.resize(0, 123);
    assert_eq!(vec, stack_vec![]);
}

#[test]
#[should_panic]
fn resize_fail() {
    let mut vec = stack_vec![0, 1, 2; cap = 5];
    vec.resize(6, 1111);
}

mod drop {
    use super::*;

    use std::cell::Cell;
    use std::rc::Rc;

    struct DropTracker {
        dropped: Rc<Cell<bool>>,
    }

    impl DropTracker {
        pub fn new() -> Self {
            Self {
                dropped: Rc::new(Cell::new(false)),
            }
        }

        pub fn dropped(&self) -> Rc<Cell<bool>> {
            Rc::clone(&self.dropped)
        }
    }

    impl Clone for DropTracker {
        fn clone(&self) -> Self {
            Self {
                dropped: Rc::new(Cell::new(self.dropped.get())),
            }
        }
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }

    fn assert_drop<const N: usize>(
        vec_len: usize,
        func: fn(StackVec<DropTracker, N>),
    ) {
        let vec: StackVec<_, N> = (0..vec_len)
            .map(|_| DropTracker::new())
            .collect();
        let tracker_refs: Vec<Rc<_>> = vec.iter()
            .map(|tracker| tracker.dropped())
            .collect();
        // do some operations on `vec` and drop it
        func(vec);
        assert!(tracker_refs.iter().all(|t| t.get()));
    }

    #[test]
    fn stack_vec() {
        fn func(mut vec: StackVec<DropTracker, 10>) {
            vec.pop();
            vec.truncate(5);
        }
        assert_drop(10, func);
    }

    #[test]
    fn into_iter() {
        fn func(vec: StackVec<DropTracker, 40>) {
            let iter = vec.into_iter();
            let _ = iter.take(4);
        }
        assert_drop::<40>(10, func);
    }
}
