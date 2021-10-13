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
fn any() {
    let mut mock = Foo::faux();
    faux::when!(mock.one_ref_arg(_)).then_return(1337);
    assert_eq!(mock.one_ref_arg(&Data { a: 2, b: 3 }), 1337);
}

#[test]
fn eq() {
    let mut mock = Foo::faux();

    let data = Data { a: 2, b: 3 };
    faux::when!(mock.one_ref_arg(data.clone())).then_return(1337);
    assert_eq!(mock.one_ref_arg(&data), 1337);
}

#[test]
fn eq_against() {
    let mut mock = Foo::faux();

    #[derive(Debug)]
    struct OtherData {
        a: i32,
        b: u32,
    }

    impl PartialEq<Data> for OtherData {
        fn eq(&self, rhs: &Data) -> bool {
            self.a == rhs.a && self.b == rhs.b
        }
    }

    faux::when!(mock.one_ref_arg(*_ == OtherData { a: 1, b: 5 })).then_return(789);

    let data = Data { a: 1, b: 5 };
    assert_eq!(mock.one_ref_arg(&data), 789);
}

#[test]
fn custom_matcher() {
    use faux::matcher::ArgMatcher;
    use std::fmt::{self, Formatter};
    let mut mock = Foo::faux();

    struct AddsToLessThan20;
    impl ArgMatcher<Data> for AddsToLessThan20 {
        fn matches(&self, arg: &Data) -> bool {
            (arg.a + arg.b as i32) < 20
        }
    }

    impl fmt::Display for AddsToLessThan20 {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "(_.a + _.b) < 20")
        }
    }

    faux::when!(mock.one_ref_arg(*_ = AddsToLessThan20)).then_return(123);

    let data = Data { a: 8, b: 11 };
    assert_eq!(mock.one_ref_arg(&data), 123);
}

#[test]
fn pattern() {
    let mut mock = Foo::faux();
    faux::when!(mock.one_ref_arg(_ = faux::pattern!(&Data => Data { a: 2, .. }))).then_return(123);
    assert_eq!(mock.one_ref_arg(&Data { a: 2, b: 789 }), 123);
}

#[test]
fn from_fn() {
    let mut mock = Foo::faux();
    faux::when!(mock.one_ref_arg(_ = faux::from_fn!(|data: &&Data| data.b == 3))).then_return(123);
    assert_eq!(mock.one_ref_arg(&Data { a: 123, b: 3 }), 123);
}

#[test]
fn mixed_args() {
    let mut mock = Foo::faux();
    let data = Data { a: 2, b: 3 };
    faux::when!(mock.two_args(_, 4)).then_return(777);
    assert_eq!(mock.two_args(&data, 4), 777);
}

#[test]
fn multiple_mocks() {
    let mut mock = Foo::faux();
    let data = Data { a: 2, b: 3 };
    faux::when!(mock.two_args).then_return(0);
    faux::when!(mock.two_args(_, 4)).then_return(1);
    faux::when!(mock.two_args(_, 10)).then_return(2);
    faux::when!(mock.two_args(data.clone(), 4)).then_return(3);

    assert_eq!(mock.two_args(&Data { a: 4, b: 3 }, 8), 0);
    assert_eq!(mock.two_args(&Data { a: 0, b: 4 }, 4), 1);
    assert_eq!(mock.two_args(&Data { a: 1, b: 4 }, 10), 2);
    assert_eq!(mock.two_args(&data, 4), 3);
}

#[test]
// #[should_panic]
fn unmatched_args() {
    let mut mock = Foo::faux();
    let data = Data { a: 2, b: 3 };
    faux::when!(mock.two_args(_, 42)).then_return(777);
    mock.two_args(&data, 2);
}
