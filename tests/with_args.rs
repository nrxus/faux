#[faux::create]
pub struct Foo {
    a: u32,
}

#[faux::methods]
impl Foo {
    pub fn new(a: u32) -> Self {
        Foo { a }
    }

    pub fn no_args(&self) -> u32 {
        self.a
    }

    pub fn one_arg(&self, a: i32) -> i32 {
        self.a as i32 + a
    }

    pub fn two_args(&self, a: i32, b: &u32) -> Vec<i32> {
        vec![self.a as i32, a, *b as i32]
    }
}

#[test]
fn success_with_args() {
    let mut mock = Foo::faux();

    faux::when!(mock.no_args).with_args(()).then(|_| 5);
    assert_eq!(mock.no_args(), 5);

    faux::when!(mock.one_arg)
        .with_args(faux::matcher::Single(faux::matcher::eq(3)))
        .then_return(10);
    assert_eq!(mock.one_arg(3), 10);

    faux::when!(mock.two_args)
        .with_args((faux::matcher::any(), faux::matcher::eq(10)))
        .then_return(vec![4]);
    assert_eq!(mock.two_args(3, &10), vec![4]);
}

#[test]
#[should_panic]
fn fail_with_args() {
    let mut mock = Foo::faux();

    faux::when!(mock.two_args)
        .with_args((faux::matcher::eq(4), faux::matcher::any()))
        .then_return(vec![2]);

    mock.two_args(3, &10);
}
