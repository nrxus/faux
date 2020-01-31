trait MyTrait {
    fn assoc();
    fn method(&self);
}

trait GenericTrait<'a, T> {
    fn g_assoc() {}
    fn g_method(&self) {}
}

#[faux::create]
struct MyStruct {}

// normal impl can be mocked
#[faux::methods]
impl MyStruct {
    fn new() -> Self {
        MyStruct {}
    }
}

// mocking trait impl works even if there is another mocked impl
#[faux::methods]
impl MyTrait for MyStruct {
    fn assoc() {}
    fn method(&self) {}
}

// mocking a second immpl trait in the same mod works
#[faux::methods]
impl<'a, T> GenericTrait<'a, T> for MyStruct {
    fn g_assoc() {}
    fn g_method(&self) {}
}

// more complicated generic stuff
#[allow(dead_code)]
#[faux::create]
struct GenericStruct<T, S> {
    t: T,
    s: S,
}

#[faux::methods]
impl<T> GenericStruct<T, i32> {
    fn make(t: T, s: i32) -> Self {
        GenericStruct { t, s }
    }
}

#[faux::methods]
impl<'a, T> GenericTrait<'a, T> for GenericStruct<T, i32>
where
    T: std::fmt::Debug,
{
    fn g_assoc() {}
    fn g_method(&self) {}
}

#[test]
fn simple_trait() {
    MyStruct::assoc();

    let my_struct = MyStruct::new();
    my_struct.method();

    let mut faux = MyStruct::faux();
    faux::when!(faux.method).safe_then(|_| {});
    faux.method();
}

#[test]
fn generic_trait() {
    GenericStruct::<String, i32>::g_assoc();

    let gen_struct = GenericStruct::make("foo", 3);
    gen_struct.g_method();

    let mut faux = GenericStruct::<&'static str, i32>::faux();
    faux::when!(faux.g_method).safe_then(|_| {});
    faux.g_method();
}
