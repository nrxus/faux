use faux;

#[faux_macros::duck]
pub struct Foo {
    a: u32,
}

#[faux_macros::quack]
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

    pub fn add_stuff_2(&self, x: i32, y: i32) -> i32 {
        self.a as i32 + x + y
    }

    pub fn some_ref(&self, x: &i32) -> i32 {
        self.a as i32 + *x
    }
}

#[test]
fn real_struct() {
    let real = Foo::new(3);
    assert_eq!(real.get_stuff(), 3);
    assert_eq!(real.add_stuff(2), 5);
}

#[test]
fn ducks_quack() {
    let mut mock = Foo::quack();
    // when!(mock.get_stuff).then(|()| 10);
    //TODO: Add macro that looks like when!(mock::get_stuff).then(|| 10);
    unsafe {
        mock._mock_once_get_stuff(|_| 10);
    }
    assert_eq!(mock.get_stuff(), 10);
}

#[test]
fn ducks_quack_arguments() {
    let mut mock = Foo::quack();
    //TODO: Add macro that looks like when!(mock::add_stuff).then(|a| 90);
    unsafe {
        mock._mock_once_add_stuff_2(|(a, _)| a);
    }
    assert_eq!(mock.add_stuff_2(90, 30), 90);
}

#[test]
fn ducks_ref_arguments() {
    let mut mock = Foo::quack();
    //TODO: Add macro that looks like when!(mock::add_stuff).then(|a| 90);
    unsafe {
        mock._mock_once_some_ref(|a| *a);
    }
    let x = 30 + 30;
    assert_eq!(mock.some_ref(&x), 60);
}

#[test]
#[should_panic]
fn ducks_panic_with_no_quacks() {
    let mock = Foo::quack();
    mock.get_stuff();
}
