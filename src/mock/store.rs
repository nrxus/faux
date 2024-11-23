use std::collections::HashMap;

use crate::InvocationError;

use super::{unchecked::Unchecked, Mock};

#[derive(Debug)]
pub struct Store<'stub> {
    pub struct_name: &'static str,
    stubs: HashMap<usize, Unchecked<'stub>>,
}

impl<'stub> Store<'stub> {
    pub fn new(struct_name: &'static str) -> Self {
        Store {
            struct_name,
            stubs: HashMap::new(),
        }
    }

    /// Returns a mutable reference to a [`Mock`] for a given function
    ///
    /// If the given function has not yet been mocked, an empty mock
    /// is created for the function.
    pub fn get_mut<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        fn_name: &'static str,
    ) -> &mut Mock<'stub, I, O> {
        let mock = self.stubs.entry(id as usize).or_insert_with(|| {
            let mock: Mock<I, O> = Mock::new(fn_name);
            mock.into()
        });

        let mock = unsafe { mock.as_typed_mut() };
        assert_name(mock, fn_name);
        mock
    }

    /// Returns a reference to a [`Mock`] for a given function
    ///
    /// `None` is returned if the function was never mocked
    pub unsafe fn get<R, I, O>(
        &self,
        id: fn(R, I) -> O,
        fn_name: &'static str,
        generics: &'static str,
    ) -> Result<&Mock<'stub, I, O>, InvocationError> {
        match self.stubs.get(&(id as usize)).map(|m| m.as_typed()) {
            Some(mock) => {
                assert_name(mock, fn_name);
                Ok(mock)
            }
            None => Err(InvocationError {
                fn_name,
                struct_name: self.struct_name,
                generics,
                stub_error: super::InvocationError::NeverStubbed,
            }),
        }
    }
}

fn assert_name<I, O>(mock: &Mock<I, O>, fn_name: &'static str) {
    assert_eq!(
        mock.name(),
        fn_name,
        "faux bug: conflicting mock names: '{}' vs '{}'",
        mock.name(),
        fn_name
    );
}
