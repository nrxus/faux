#[faux::create]
#[derive(Debug)]
pub struct Foo {
    a: u32,
}

#[faux::methods]
impl Foo {
    pub fn new(a: u32) -> Self {
        Foo { a }
    }

    pub fn no_args(&self, _: i32) -> u32 {
        todo!()
    }
}

#[test]
fn simple_expect() {
    let mut mock = Foo::faux();
    faux::when!(mock.no_args).then_return(5);
    // faux::expect!(mock.no_args);
    faux::expect!(mock.no_args(5));
    faux::expect!(mock.no_args(3));
    // mock.no_args(5);
}
