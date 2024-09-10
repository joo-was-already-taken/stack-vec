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
    type Sv = StackVec<i32, 4>;

    let mut vec = Sv::new();
    vec.push(0);
    assert_eq!(vec, Sv::from_array([0]).unwrap());
    vec.push(1);
    assert_eq!(vec, Sv::from_array([0, 1]).unwrap());
    let push_res = vec.try_push(2);
    assert_eq!(vec, Sv::from_array([0, 1, 2]).unwrap());
    assert_eq!(push_res, Ok(()));
    vec.push(3);
    assert_eq!(vec, Sv::from_array([0, 1, 2, 3]).unwrap());
    assert_eq!(vec.try_push(4), Err(NotEnoughSpaceError));
}

#[test]
fn push_zst() {
    #[derive(Clone, Copy)]
    struct Zst;

    let mut vec = StackVec::from([Zst; 11]);
    assert_eq!(vec.len(), 11);
    assert_eq!(vec.try_push(Zst), Err(NotEnoughSpaceError));
    assert_eq!(vec.len(), 11);
}

// #[test]
// fn insert() {
//     todo!()
// }
