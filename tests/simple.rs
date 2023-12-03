#[faux::create]
pub struct Foo {
    a: u32,
}

#[faux::methods]
impl Foo {
    pub fn new(a: u32) -> Self {
        Foo { a }
    }

    pub fn get_stuff(&self) -> u32 {
        self.a
    }

    pub fn add_stuff(&self, x: i32) -> i32 {
        self.a as i32 + x
    }

    pub fn add_stuff_2(&self, x: i32, y: &i32) -> i32 {
        self.a as i32 + x + y
    }

    pub fn ret_ref(&self, _: &u32) -> &u32 {
        &self.a
    }

    fn private(&self, _: &u32) -> &u32 {
        &self.a
    }
}

fn load_a() -> Result<u32, Box<dyn std::error::Error>> {
    Ok(3)
}

// tests that functions not tagged by `faux::methods` can use the ones
// that are in a `faux::methods` impl block
impl Foo {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let a = load_a()?;
        Ok(Foo::new(a))
    }
}

#[test]
fn real_struct() {
    let real = Foo::new(3);
    assert_eq!(real.get_stuff(), 3);
    assert_eq!(real.add_stuff(2), 5);
}

#[test]
fn faux_single_arg() {
    let mut mock = Foo::faux();
    faux::when!(mock.get_stuff).then(|_| 10);
    assert_eq!(mock.get_stuff(), 10);
}

#[test]
fn faux_multi_arg() {
    let mut mock = Foo::faux();
    faux::when!(mock.add_stuff_2).then(|(a, &b)| a - b);
    assert_eq!(mock.add_stuff_2(90, &30), 60);
}

#[test]
fn faux_ref_output() {
    let mut mock = Foo::faux();
    unsafe { faux::when!(mock.ret_ref).then_unchecked(|a| a) };
    let x = 30 + 30;
    assert_eq!(*mock.ret_ref(&x), 60);
}

#[test]
#[should_panic]
fn unmocked_faux_panics() {
    let mock = Foo::faux();
    mock.get_stuff();
}
