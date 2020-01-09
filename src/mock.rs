#[doc(hidden)]
pub struct Mock(Box<dyn FnOnce(()) -> () + Send>);

impl std::fmt::Debug for Mock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("mock")
    }
}

impl Mock {
    pub(crate) unsafe fn new<I, O>(mock: impl FnOnce(I) -> O + Send) -> Self {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = std::mem::transmute(mock);
        Mock(mock)
    }
}

impl Mock {
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
