use std::any::Any;

#[doc(hidden)]
pub enum Mock {
    OnceUnsafe(UnsafeMock),
    OnceSafe(SafeMock),
}

impl Mock {
    pub(crate) fn safe<I: 'static, O: 'static>(mock: impl FnOnce(I) -> O + 'static) -> Self {
        let mock = |input: Box<dyn Any>| {
            let input = *(input.downcast().unwrap());
            let output = mock(input);
            Box::new(output) as Box<dyn Any>
        };
        Mock::OnceSafe(SafeMock(Box::new(mock)))
    }

    pub(crate) unsafe fn r#unsafe<I, O>(mock: impl FnOnce(I) -> O) -> Self {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = std::mem::transmute(mock);
        Mock::OnceUnsafe(UnsafeMock(mock))
    }
}

#[doc(hidden)]
pub struct SafeMock(Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>);

impl SafeMock {
    pub fn call<I: 'static, O: 'static>(self, input: I) -> O {
        let input = Box::new(input) as Box<dyn Any>;
        *self.0(input).downcast().unwrap()
    }
}

#[doc(hidden)]
pub struct UnsafeMock(Box<dyn FnOnce(()) -> ()>);

impl UnsafeMock {
    /// # Safety
    ///
    /// [#[methods]](methods) makes sure this function is
    /// called correctly when the method is invoked.  Do not use this
    /// function directly.
    pub unsafe fn call<I, O>(self, input: I) -> O {
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(self.0);
        mock(input)
    }
}
