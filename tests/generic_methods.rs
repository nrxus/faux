#[faux::create]
struct Foo {}

#[faux::methods]
impl Foo {
    pub fn gen_method<Q, F: Default>(&self, _: Q) -> F {
        F::default()
    }

    pub fn callback<F>(&self, f: F) -> u32
    where
        F: Fn(i32) -> u32,
    {
        f(3)
    }
}

use faux::when;

#[test]
fn gen_method() {
    let mut foo = Foo::faux();
    when!(foo.gen_method).safe_then(|a: u32| a);
    assert_eq!(foo.gen_method::<_, u32>(20), 20);
}

#[test]
fn callback() {
    let mut foo = Foo::faux();
    when!(foo.callback).safe_then(|a| 10);
    assert_eq!(foo.callback(|a| a as u32), 10);
}
