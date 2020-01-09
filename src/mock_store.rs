use crate::Mock;
use std::collections::HashMap;

#[doc(hidden)]
/// ```
/// fn implements_sync<T: Sync>(_: T) {}
///
/// implements_sync(3);
/// implements_sync(faux::MaybeFaux::Real(3));
/// ```
///
/// ```
/// fn implements_debug<T: std::fmt::Debug>(_: T) {}
///
/// implements_debug(3);
/// implements_debug(faux::MaybeFaux::Real(3));
/// ```
#[derive(Debug)]
pub enum MaybeFaux<T> {
    Real(T),
    Faux(std::sync::Mutex<MockStore>),
}

impl<T> MaybeFaux<T> {
    pub fn faux() -> Self {
        MaybeFaux::Faux(std::sync::Mutex::new(MockStore::new()))
    }
}

/// Stores the mocked methods
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: HashMap<&'static str, Mock>,
}

#[doc(hidden)]
impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: HashMap::new(),
        }
    }

    /// # Safety
    ///
    /// Do not use this function without going through [When].  It is
    /// the caller's responsability to pass a mock that is safe.
    ///
    /// [When]: When
    pub unsafe fn unsafe_mock_once<I, O>(
        &mut self,
        id: &'static str,
        mock: impl FnOnce(I) -> O + Send,
    ) {
        self.mocks.insert(id, Mock::new(mock));
    }

    pub fn mock_once<I, O>(&mut self, id: &'static str, mock: impl FnOnce(I) -> O + 'static + Send)
    where
        I: 'static,
        O: 'static,
    {
        unsafe {
            self.mocks.insert(id, Mock::new(mock));
        }
    }

    pub fn get_mock(&mut self, id: &str) -> Option<Mock> {
        self.mocks.remove(id)
    }
}
