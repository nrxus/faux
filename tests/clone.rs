#![allow(clippy::redundant_clone)]

#[faux::create]
#[derive(Clone)]
pub struct Foo {
    a: i32,
}

#[faux::methods]
impl Foo {
    pub fn new(a: i32) -> Self {
        Foo { a }
    }

    pub fn get(&self) -> i32 {
        self.a
    }
}

#[test]
fn can_clone_real() {
    let real = Foo::new(3);
    let cloned = real.clone();
    assert_eq!(cloned.get(), 3);
}

#[test]
fn can_clone_mock() {
    let mut mock = Foo::faux();
    let cloned = mock.clone();
    faux::when!(mock.get()).then_return(4);
    assert_eq!(cloned.get(), 4);
}
