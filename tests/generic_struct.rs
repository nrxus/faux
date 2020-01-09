#[faux::create]
pub struct Generic<'a, T, R> {
    a: T,
    b: i32,
    c: &'a R,
}

#[faux::methods]
impl<'a, T, R> Generic<'a, T, R> {
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

#[test]
fn real() {
    let hello = "hello".to_string();
    let real = Generic::new(10, &hello);
    assert_eq!(real.get(), &20);
    assert_eq!(real.get_ref(), &10);
    assert_eq!(real.life_ref(), &hello);
}

#[test]
fn mocked() {
    use faux::when;

    let mut fake: Generic<String, u32> = Generic::faux();
    unsafe { when!(fake.get).then(|_| &5) }
    assert_eq!(fake.get(), &5);

    let goodbye = "goodbye".to_string();
    unsafe { when!(fake.get_ref).then(|_| &goodbye) }
    assert_eq!(fake.get_ref(), &goodbye);

    unsafe { when!(fake.life_ref).then(|_| &2) }
    assert_eq!(fake.life_ref(), &2);
}
