#![allow(clippy::blacklisted_name)]

pub trait MyTrait {}

#[faux::create]
pub struct Foo {}

#[faux::methods]
impl Foo {
    pub fn foo(&self, _: impl Fn(i32) -> i32) -> i32 {
        todo!()
    }

    pub fn bar(&self, _: &impl MyTrait) -> u8 {
        todo!()
    }
}

#[test]
fn generic() {
    let mut foo = Foo::faux();
    faux::when!(foo.foo).then(|add_one| add_one(2) + 5);
    assert_eq!(foo.foo(|i| i + 1), 8);
}

#[test]
fn generic_reference() {
    struct MyStruct {}
    impl MyTrait for MyStruct {}

    let mut foo = Foo::faux();
    faux::when!(foo.bar).then_return(3);
    assert_eq!(foo.bar(&MyStruct {}), 3);
}
