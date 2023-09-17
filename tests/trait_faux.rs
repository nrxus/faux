#[faux::faux]
pub trait MyTrait {
    fn my_func(a: i32, b: u64) -> String
    where
        Self: Sized;
    fn my_method(&self, a: &str) -> u32;
}

// #[faux::faux]
// pub trait MyGenericTrait<'a, A, B> {
// }

#[test]
fn can_mock_traits() {
    let mut faux = MyTrai::tfaux();
    faux::when!(faux.my_method("hello")).then_return(5);
    assert_eq!(faux.my_method("hello"), 5);
}

#[test]
fn panics_on_static_methods() {
    let faux = <dyn MyTrait>::faux();
    fn call<F: MyTrait>(_: F) {
        F::my_func(2, 3);
    }
    call(faux);
}
