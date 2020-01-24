mod bar {
    #[faux::create]
    pub struct Bar {
        a: i32,
    }

    #[faux::methods]
    impl Bar {
        pub fn new() -> Bar {
            Bar { a: 10 }
        }

        pub fn get(&self) -> i32 {
            self.a
        }

        pub fn num() -> i32 {
            4
        }
    }
}

mod foo {
    #[faux::create]
    pub struct Foo {
        f: &'static str,
    }

    #[faux::methods]
    impl Foo {
        pub fn new(f: &'static str) -> Self {
            Foo { f }
        }

        pub fn get(&self) -> &'static str {
            self.f
        }
    }
}

mod other {
    use crate::foo;

    #[faux::methods(path = "crate")]
    impl foo::Foo {
        pub fn other_new() -> foo::Foo {
            foo::Foo::new("hello")
        }

        pub fn get_chunk(&self, chunk: usize) -> &'static str {
            &self.get()[0..chunk]
        }
    }
}

#[faux::methods]
impl bar::Bar {
    pub fn add(&self) -> i32 {
        self.get() + bar::Bar::num()
    }
}

#[test]
fn real() {
    use crate::bar::Bar;
    use crate::foo::Foo;

    let foo = Foo::other_new();
    assert_eq!(foo.get_chunk(1), "h");

    let bar = Bar::new();
    assert_eq!(bar.add(), 14);
}

#[test]
fn mocked() {
    use crate::{bar::Bar, foo::Foo};
    use faux::when;

    let mut foo = Foo::faux();
    when!(foo.get_chunk).safe_then(|_| "hello");
    assert_eq!(foo.get_chunk(1), "hello");

    let mut bar = Bar::faux();
    when!(bar.add).safe_then(|_| 3);
    assert_eq!(bar.add(), 3);
}
