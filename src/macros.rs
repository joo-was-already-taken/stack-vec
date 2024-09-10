#[macro_export]
macro_rules! stack_vec {
    () => {
        $crate::StackVec::new()
    };
    ($($elem:expr),+ $(,)?) => {
        $crate::StackVec::from([$($elem),*])
    };
    ($elem:expr; $length:expr) => {
        // $crate::StackVec::from([$elem; $length])
        $crate::StackVec::from_elem($elem, $length).unwrap()
    };
}

#[cfg(test)]
#[test]
fn initialization() {
    use crate::StackVec;
    assert_eq!(stack_vec![], StackVec::<i32, 6>::new());
    assert_eq!(stack_vec![4, 3, 2, 1], StackVec::from([4, 3, 2, 1]));
    assert_eq!(stack_vec![4, 3, 2, 1,], StackVec::from([4, 3, 2, 1]));
    assert_eq!(stack_vec![69; 7], StackVec::from([69; 7]));
}
