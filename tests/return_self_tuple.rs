use std::sync::Arc;
use std::rc::Rc;
use std::result::Result;
use std::error::Error;

#[faux::create]
pub struct Foo;

#[faux::methods]
impl Foo {
    pub fn self_self() -> (Self, Self) {
        (Foo, Foo)
    }

    pub fn self_path() -> (Self, i32) {
        (Foo, 1)
    }

    pub fn self_box() -> (Self, Box<Self>) {
        (Foo, Box::new(Foo))
    }

    pub fn self_rc() -> (Self, Rc<Self>) {
        (Foo, Rc::new(Foo))
    }

    pub fn self_arc() -> (Self, Arc<Self>) {
        (Foo, Arc::new(Foo))
    }

    pub fn self_result() -> (Self, Result<Self, Box<dyn Error>>) {
        (Foo, Ok(Foo))
    }

    pub fn self_option() -> (Self, Option<Self>) {
        (Foo, Some(Foo))
    }
}
