use std::collections::HashMap;

use super::{unchecked::Unchecked, Mock};

#[derive(Debug, Default)]
pub struct Store {
    stubs: HashMap<usize, Unchecked<'static>>,
}

impl Store {
    /// Returns a mutable reference to a [`Mock`] for a given function
    ///
    /// If the given function has not yet been mocked, an empty mock
    /// is created for the function.
    pub fn get_or_create<R, I, O>(&mut self, id: fn(R, I) -> O) -> &mut Mock<I, O> {
        let mock = self.stubs.entry(id as usize).or_insert_with(|| {
            let mock: Mock<I, O> = Mock::new();
            mock.into()
        });

        unsafe { mock.as_typed_mut() }
    }

    /// Returns a reference to a [`Mock`] for a given function
    ///
    /// `None` is returned if the function was never mocked
    pub unsafe fn get<R, I, O>(&self, id: fn(R, I) -> O) -> Option<&Mock<I, O>> {
        self.stubs.get(&(id as usize)).map(|m| m.as_typed())
    }
}
