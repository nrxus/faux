#![allow(clippy::disallowed_names)]

pub trait MyTrait {
    fn x(&self) -> u32;
}

pub trait MyOtherTrait {
    fn x(&self) -> u32;
}

impl<T: ?Sized + MyTrait> MyTrait for Box<T> {
    fn x(&self) -> u32 {
        self.as_ref().x()
    }
}

impl<T: ?Sized + MyOtherTrait> MyOtherTrait for Box<T> {
    fn x(&self) -> u32 {
        self.as_ref().x()
    }
}

struct MyStruct {}
impl MyTrait for MyStruct {
    fn x(&self) -> u32 {
        todo!()
    }
}

impl MyOtherTrait for MyStruct {
    fn x(&self) -> u32 {
        todo!()
    }
}

#[faux::create]
pub struct Foo {}

#[faux::methods]
impl Foo {
    pub fn foo(&self, _: impl Fn(i32) -> i32) -> i32 {
        todo!()
    }

    pub fn bar(&self, _: &impl MyTrait) -> u8 {
        todo!()
    }

    pub fn gen_params<F: ?Sized>(&self, _: &F) -> String {
        todo!()
    }

    pub fn gen_lifes<'a, 'b: 'a>(&self, _: &'b i32) -> &'a i32 {
        todo!()
    }

    pub fn gen_output<T>(&self) -> T {
        todo!()
    }

    pub fn impl_out_impl_in(&self, _: &impl MyTrait) -> impl MyOtherTrait + '_ {
        MyStruct {}
    }

    pub fn impl_out(&self) -> impl MyTrait {
        MyStruct {}
    }
}

#[test]
fn impl_generic() {
    let mut foo = Foo::faux();
    faux::when!(foo.foo).then(|add_one| add_one(2) + 5);
    assert_eq!(foo.foo(|i| i + 1), 8);
}

#[test]
fn impl_generic_reference() {
    struct MyStruct {}
    impl MyTrait for MyStruct {
        fn x(&self) -> u32 {
            todo!()
        }
    }

    let mut foo = Foo::faux();
    faux::when!(foo.bar).then_return(3);
    assert_eq!(foo.bar(&MyStruct {}), 3);
}

#[test]
fn generic_methods() {
    let mut foo = Foo::faux();

    faux::when!(foo.gen_params::<i32>(_)).then_return("any int".to_owned());
    faux::when!(foo.gen_params::<i32>(4)).then_return("exactly 4".to_owned());
    faux::when!(foo.gen_params::<str>("hello")).then_return("exactly hello".to_owned());
    assert_eq!(foo.gen_params(&2), "any int");
    assert_eq!(foo.gen_params(&3), "any int");
    assert_eq!(foo.gen_params(&4), "exactly 4");
    assert_eq!(foo.gen_params("hello"), "exactly hello");
}

#[test]
#[should_panic]
fn generic_methods_wrong_type() {
    let mut foo = Foo::faux();

    faux::when!(foo.gen_params::<i32>(_)).then_return("xyz".to_owned());
    foo.gen_params("some string");
}

#[test]
fn generic_lifetimes() {
    let mut foo = Foo::faux();

    faux::when!(foo.gen_lifes).then_return(&5);
    faux::when!(foo.gen_lifes(&2)).then(|x| x);
    assert_eq!(foo.gen_lifes(&2), &2);
    assert_eq!(foo.gen_lifes(&3), &5);
    assert_eq!(foo.gen_lifes(&4), &5);
}

#[test]
fn generic_output() {
    let mut foo = Foo::faux();
    faux::when!(foo.gen_output).then_return(32_i32);
    assert_eq!(foo.gen_output::<i32>(), 32);
}

struct MyTestStruct(u32);

impl MyTrait for MyTestStruct {
    fn x(&self) -> u32 {
        self.0
    }
}

impl MyOtherTrait for MyTestStruct {
    fn x(&self) -> u32 {
        self.0
    }
}

#[test]
fn impl_output() {
    let mut foo = Foo::faux();
    faux::when!(foo.impl_out)
        .once()
        .then_return(Box::new(MyTestStruct(42)));
    assert_eq!(foo.impl_out().x(), 42);
}

#[test]
fn impl_output_with_gen_input() {
    let mut foo = Foo::faux();
    faux::when!(foo.impl_out_impl_in)
        .once()
        .then_return(Box::new(MyTestStruct(2)));
    assert_eq!(foo.impl_out_impl_in(&MyStruct {}).x(), 2);
}
