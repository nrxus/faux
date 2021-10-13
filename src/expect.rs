use std::marker::PhantomData;

use crate::{
    matcher::{AnyInvocation, InvocationMatcher},
    mock_store::SavedExpectation,
    MockStore,
};

pub struct Expect<'m, I> {
    expectation: &'m mut SavedExpectation,
    _marker: PhantomData<fn(I) -> ()>,
}

impl<'m, I> Expect<'m, I> {
    #[doc(hidden)]
    pub fn new<R, O>(id: fn(R, I) -> O, store: &'m mut MockStore, fn_name: &'static str) -> Self {
        let expectation = store.expect(id, fn_name, Box::new(AnyInvocation));

        Expect {
            expectation,
            _marker: PhantomData,
        }
    }

    pub fn with_args(self, matcher: impl InvocationMatcher<I> + Send + 'static) -> Self {
        let matcher = Box::new(matcher) as Box<dyn InvocationMatcher<I> + Send>;
        *self.expectation = matcher.into();

        self
    }
}
