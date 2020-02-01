use crate::mock::{MockTimes, ReturnedMock, StoredMock};
use std::collections::{self, HashMap};

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
    mocks: HashMap<&'static str, StoredMock>,
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
        self.mocks.insert(id, StoredMock::once(mock));
    }

    pub fn mock_once<I, O>(&mut self, id: &'static str, mock: impl FnOnce(I) -> O + 'static + Send)
    where
        I: 'static,
        O: 'static,
    {
        unsafe {
            self.mocks.insert(id, StoredMock::once(mock));
        }
    }

    /// # Safety
    ///
    /// Do not use this function without going through [When].  It is
    /// the caller's responsability to pass a mock that is safe.
    ///
    /// [When]: When
    pub unsafe fn unsafe_mock<I, O>(
        &mut self,
        id: &'static str,
        mock: impl FnMut(I) -> O + Send,
        times: MockTimes,
    ) {
        self.mocks.insert(id, StoredMock::many(mock, times));
    }

    pub fn mock<I, O>(
        &mut self,
        id: &'static str,
        mock: impl FnMut(I) -> O + 'static + Send,
        times: MockTimes,
    ) where
        I: 'static,
        O: 'static,
    {
        unsafe {
            self.unsafe_mock(id, mock, times);
        }
    }

    pub fn get_mock(&mut self, id: &'static str) -> Option<ReturnedMock> {
        match self.mocks.entry(id) {
            // no mock stored
            collections::hash_map::Entry::Vacant(_) => None,
            collections::hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                // we did not remove the mock on its "last" possible call so remove it now
                // we do this because ReturnedMock::Many wants a reference
                // so the mock must still live somewhere even during its last call
                StoredMock::Many(_, MockTimes::Times(0)) => {
                    entry.remove();
                    None
                }
                // only a single mock
                // remove and return mock
                StoredMock::Once(_) => {
                    let mock = entry.remove();
                    match mock {
                        StoredMock::Once(mock) => Some(ReturnedMock::Once(mock)),
                        StoredMock::Many(_, _) => unreachable!(),
                    }
                }
                // mock that can be called multiple times
                // return mock but do not remove it
                // call into_mut and stop using the get_mut because of lifetime shenanigans
                StoredMock::Many(_, _) => match entry.into_mut() {
                    StoredMock::Many(many, times) => {
                        times.decrement();
                        // One may think that doing &mut *many is the same thing
                        // but NO IT IS NOT Why? ¯\_(ツ)_/¯
                        // Be very careful because I do not understand why
                        // and the other way fails some tests... but not all?
                        Some(ReturnedMock::Many(many.as_mut()))
                    }
                    _ => unreachable!(),
                },
            },
        }
    }
}
