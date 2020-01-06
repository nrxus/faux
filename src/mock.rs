use std::any::Any;

#[doc(hidden)]
pub enum Mock {
    OnceUnsafe(UnsafeMock),
    OnceSafe(SafeMock),
}

impl std::fmt::Debug for Mock {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mock::OnceUnsafe(_) => fmt.write_str("unsafe mock"),
            Mock::OnceSafe(_) => fmt.write_str("safe mock"),
        }
    }
}

impl Mock {
    pub(crate) fn safe<I: 'static, O: 'static>(mock: impl FnOnce(I) -> O + 'static + Send) -> Self {
        let mock = |input: BoxedAny| {
            let input = *(input.downcast().unwrap());
            let output = mock(input);
            Box::new(output) as BoxedAny
        };
        Mock::OnceSafe(SafeMock(Box::new(mock)))
    }

    pub(crate) unsafe fn r#unsafe<I, O>(mock: impl FnOnce(I) -> O + Send) -> Self {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = std::mem::transmute(mock);
        Mock::OnceUnsafe(UnsafeMock(mock))
    }
}

#[doc(hidden)]
pub struct SafeMock(Box<dyn FnOnce(BoxedAny) -> BoxedAny + Send>);

impl SafeMock {
    pub fn call<I: 'static, O: 'static>(self, input: I) -> O {
        let input = Box::new(input) as BoxedAny;
        *self.0(input).downcast().unwrap()
    }
}

#[doc(hidden)]
pub struct UnsafeMock(Box<dyn FnOnce(()) -> () + Send>);

impl UnsafeMock {
    /// # Safety
    ///
    /// [#[methods]] makes sure this function is called correctly when
    /// the method is invoked.  Do not use this function directly.
    ///
    /// [#\[methods\]]: methods
    pub unsafe fn call<I, O>(self, input: I) -> O {
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(self.0);
        mock(input)
    }
}

type BoxedAny = Box<dyn Any>;
