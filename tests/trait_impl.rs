trait MyTrait {
    fn assoc();
    fn method(&self);
}

trait GenericTrait<'a, T> {
    fn g_assoc() {}
    fn g_method(&self) {}
}

#[faux::create]
struct MyStruct;

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
#[faux::create]
struct GenericStruct<T, S>(T, S);

#[faux::methods]
impl<'a, T> GenericTrait<'a, T> for GenericStruct<T, i32> where T: std::fmt::Debug {
    fn g_assoc() {}
    fn g_method(&self) {}
}
