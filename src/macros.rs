#[macro_export]
macro_rules! stack_vec {
    () => {
        $crate::StackVec::new()
    };
    ($($elem:expr),+ $(,)?) => {
        $crate::StackVec::from_array([$($elem),*]).unwrap()
    };
    ($($elem:expr),*; cap = $cap:expr) => {
        $crate::StackVec::<_, $cap>::from_array([$($elem),*]).unwrap()
    };
    ($elem:expr; $length:expr) => {
        $crate::StackVec::from_elem($elem, $length).unwrap()
    };
    ($elem:expr; $length:expr; cap = $cap:expr) => {
        // $crate::StackVec::from_elem($elem, $length).unwrap()
        {
            let mut vec = $crate::StackVec::<_, $cap>::new();
            for _ in 0..$length {
                vec.push($elem);
            }
            vec
        }
    };
}

#[cfg(test)]
#[test]
fn initialization() {
    use crate::StackVec;
    assert_eq!(stack_vec![], StackVec::<i32, 6>::new());
    assert_eq!(stack_vec![4, 3, 2, 1], StackVec::from([4, 3, 2, 1]));
    assert_eq!(stack_vec![4, 3, 2, 1,], StackVec::from([4, 3, 2, 1]));
    assert_eq!(stack_vec![3, 2, 1; cap = 5], StackVec::<_, 5>::from_array([3, 2, 1]).unwrap());
    assert_eq!(stack_vec![69; 7], StackVec::from([69; 7]));
}
