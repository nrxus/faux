use crate::Mock;
use std::{any::TypeId, cell::RefCell, collections::HashMap};

#[doc(hidden)]
pub enum MaybeFaux<T> {
    Real(T),
    Faux(RefCell<MockStore>),
}

impl<T> MaybeFaux<T> {
    pub fn faux() -> Self {
        MaybeFaux::Faux(RefCell::new(MockStore::new()))
    }
}

/// Stores the mocked methods
#[derive(Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: HashMap<TypeId, Mock>,
}

#[doc(hidden)]
impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: HashMap::new(),
        }
    }

    /// # Safety
    /// Do not use this function without going through [When](When).
    /// It is the caller's responsability to pass a mock that is safe.
    pub unsafe fn unsafe_mock_once<I, O>(&mut self, id: TypeId, mock: impl FnOnce(I) -> O) {
        self.mocks.insert(id, Mock::r#unsafe(mock));
    }

    pub fn mock_once<I, O>(&mut self, id: TypeId, mock: impl FnOnce(I) -> O + 'static)
    where
        I: 'static,
        O: 'static,
    {
        self.mocks.insert(id, Mock::safe(mock));
    }

    pub fn get_mock(&mut self, id: TypeId) -> Option<Mock> {
        self.mocks.remove(&id)
    }
}
