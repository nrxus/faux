#![allow(clippy::disallowed_names)]

#[faux::create]
struct Foo {
    a: i32,
}

#[faux::methods]
impl Foo {
    pub fn get(&self) -> i32 {
        self.a
    }
}

#[test]
fn always() {
    let mut foo = Foo::faux();
    faux::when!(foo.get).then(|_| 3);
    for _ in 0..20 {
        assert_eq!(foo.get(), 3);
    }
}

#[test]
fn limited() {
    let mut foo = Foo::faux();
    faux::when!(foo.get).times(3).then(|_| 3);
    for _ in 0..3 {
        assert_eq!(foo.get(), 3);
    }
}

#[test]
#[should_panic]
fn limited_past_limit() {
    let mut foo = Foo::faux();
    faux::when!(foo.get).times(3).then(|_| 3);
    for _ in 0..3 {
        foo.get();
    }
    foo.get(); // panics here
}

#[test]
fn once() {
    let mut foo = Foo::faux();
    faux::when!(foo.get).once().then(|_| 3);
    assert_eq!(foo.get(), 3);
}

#[test]
#[should_panic]
fn once_past_limit() {
    let mut foo = Foo::faux();
    faux::when!(foo.get).once().then(|_| 3);
    foo.get();
    foo.get(); //panics here
}
