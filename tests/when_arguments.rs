#[faux::create]
pub struct Foo {
    a: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Data {
    a: i32,
    b: u32,
}

#[faux::methods]
impl Foo {
    pub fn new(a: u32) -> Self {
        Foo { a }
    }

    pub fn no_args(&self) -> u32 {
        self.a
    }

    pub fn one_ref_arg(&self, data: &Data) -> u32 {
        data.b + self.a
    }

    pub fn two_args(&self, data: &Data, b: i32) -> u32 {
        data.b + self.a * b as u32
    }
}

#[derive(Debug)]
struct Bar(i32);

#[test]
fn no_args() {
    let mut mock = Foo::faux();
    faux::when!(mock.no_args()).then_return(10);
}

#[test]
fn any_arg() {
    let mut mock = Foo::faux();
    faux::when!(mock.no_args()).then_return(10);

    faux::when!(mock.one_ref_arg(_)).then_return(1337);
    assert_eq!(mock.one_ref_arg(&Data { a: 2, b: 3 }), 1337);
}

#[test]
fn eq_arg() {
    let mut mock = Foo::faux();
    faux::when!(mock.no_args()).then_return(10);

    let data = Data { a: 2, b: 3 };
    faux::when!(mock.one_ref_arg(&data.clone())).then_return(1337);
    assert_eq!(mock.one_ref_arg(&data), 1337);
}

#[test]
fn mixed_args() {
    let mut mock = Foo::faux();
    faux::when!(mock.no_args()).then_return(10);

    let data = Data { a: 2, b: 3 };
    faux::when!(mock.two_args(_, 4)).then_return(777);
    assert_eq!(mock.two_args(&data, 4), 777);
}

#[test]
#[should_panic]
fn unmatched_args() {
    let mut mock = Foo::faux();
    let data = Data { a: 2, b: 3 };
    faux::when!(mock.two_args(_, 4)).then_return(777);
    mock.two_args(&data, 2);
}
