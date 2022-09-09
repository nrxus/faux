#[faux::create]
pub struct Generic<'a, T: std::fmt::Debug, R> {
    a: T,
    b: i32,
    c: &'a R,
}

#[faux::methods]
impl<'a, T: std::fmt::Debug, R> Generic<'a, T, R> {
    pub fn new(a: T, c: &'a R) -> Self {
        Generic { a, c, b: 20 }
    }

    pub fn other_new(a: T, c: &'a R) -> Generic<'a, T, R> {
        Generic { a, c, b: 10 }
    }

    pub fn get(&self) -> &i32 {
        &self.b
    }

    pub fn get_ref(&self) -> &T {
        &self.a
    }

    pub fn life_ref(&self) -> &R {
        self.c
    }
}

#[faux::methods]
impl<'a, R> Generic<'a, i32, R> {
    pub fn get_i32(&self) -> i32 {
        self.a
    }
}

#[test]
fn real() {
    let hello = "hello".to_string();
    let real = Generic::new(10, &hello);
    assert_eq!(real.get(), &20);
    assert_eq!(real.get_ref(), &10);
    assert_eq!(real.life_ref(), &hello);
    assert_eq!(real.get_i32(), 10);
}

#[test]
fn mocked() {
    let mut fake: Generic<String, u32> = Generic::faux();
    unsafe { faux::when!(fake.get).then_unchecked(|_| &5) }
    assert_eq!(fake.get(), &5);

    let goodbye = "goodbye".to_string();
    unsafe { faux::when!(fake.get_ref).then_unchecked(|_| &goodbye) }
    assert_eq!(fake.get_ref(), &goodbye);

    unsafe { faux::when!(fake.life_ref).then_unchecked(|_| &2) }
    assert_eq!(fake.life_ref(), &2);

    let mut int_fake: Generic<i32, u32> = Generic::faux();
    faux::when!(int_fake.get_i32()).then_return(4);
    assert_eq!(int_fake.get_i32(), 4);
}
