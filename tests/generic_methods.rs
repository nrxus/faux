#![allow(clippy::blacklisted_name)]

#[faux::create]
pub struct Foo {}

#[faux::methods]
impl Foo {
    pub fn foo(&self, _: impl Fn(i32) -> i32) -> i32 {
        todo!()
    }
}

#[test]
fn generic() {
    let mut foo = Foo::faux();
    faux::when!(foo.foo).then(|add_one| add_one(2) + 5);
    assert_eq!(foo.foo(|i| i + 1), 8);
}
