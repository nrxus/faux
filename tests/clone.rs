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
fn can_call_cloned_mock() {
    let mut mock = Foo::faux();
    faux::when!(mock.get()).then_return(4);

    let cloned = mock.clone();
    assert_eq!(cloned.get(), 4);
    assert_eq!(mock.get(), 4);
}

#[test]
#[should_panic]
fn panics_when_mocking_clone() {
    let mock = Foo::faux();
    let mut clone = mock.clone();
    faux::when!(clone.get()).then_return(4);
}
