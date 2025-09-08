#![allow(clippy::redundant_clone)]

use std::{rc::Rc, sync::Arc};

type Result<T> = std::result::Result<T, ()>;

#[faux::create]
#[derive(Debug)]
pub struct Foo {
    a: i32,
}

pub struct FooError {}

#[faux::methods]
impl Clone for Foo {
    fn clone(&self) -> Self {
        Self { a: self.a }
    }
}

#[faux::methods]
impl Foo {
    pub fn new(a: i32) -> Self {
        Foo { a }
    }

    pub fn get(&self) -> i32 {
        self.a
    }

    pub fn create_similar(&self) -> Self {
        Foo { a: self.a + 1 }
    }

    pub fn new_boxed() -> Box<Self> {
        Box::new(Foo { a: 0 })
    }

    pub fn new_rc() -> Rc<Self> {
        Rc::new(Foo { a: 1 })
    }

    pub fn new_arc() -> Arc<Self> {
        Arc::new(Foo { a: 2 })
    }

    pub fn new_result() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Foo { a: 3 })
    }

    pub fn new_option() -> Option<Self> {
        Some(Foo { a: 2 })
    }

    pub fn new_tuple() -> (Self, i32) {
        (Foo { a: 4 }, 4)
    }

    pub fn new_tuple_multiple() -> (Self, i32, Self) {
        (Foo { a: 4 }, 4, Foo { a: 8 })
    }

    pub fn new_tuple_boxed() -> (Self, Box<Foo>) {
        (Foo { a: 4}, Box::new(Foo { a: 8 }))
    }

    pub fn similar_name() -> FooError {
        FooError {}
    }

    #[allow(clippy::result_unit_err)]
    pub fn new_aliased_result() -> Result<Self> {
        Ok(Foo { a: 0 })
    }
}

#[test]
fn clone_real_instances() {
    let real = Foo::new(3);
    let cloned = real.clone();
    assert_eq!(cloned.get(), 3);
}

#[test]
fn clone_mock_instances() {
    let mut mock = Foo::faux();
    faux::when!(mock.get()).then_return(20);

    faux::when!(mock.clone()).then(|_| {
        let mut other_mock = Foo::faux();
        faux::when!(other_mock.get()).then_return(30);
        other_mock
    });

    let cloned = mock.clone();

    assert_eq!(cloned.get(), 30);

    faux::when!(mock.clone()).then_return(Foo::new(5));
    let cloned = mock.clone();

    assert_eq!(cloned.get(), 5);
}

#[test]
fn create_from_real_instances() {
    let real = Foo::new(3);
    let similar = real.create_similar();
    assert_eq!(similar.get(), 4);
}

#[test]
fn create_from_mock_instances() {
    let mut mock = Foo::faux();
    faux::when!(mock.create_similar()).then(|_| {
        let mut other_mock = Foo::faux();
        faux::when!(other_mock.get()).then_return(99);
        other_mock
    });

    let similar = mock.create_similar();
    assert_eq!(similar.get(), 99);

    faux::when!(mock.create_similar()).then_return(Foo::new(5));
    let similar = mock.create_similar();
    assert_eq!(similar.get(), 5);
}
