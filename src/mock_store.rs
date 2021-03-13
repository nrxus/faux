use crate::mock::{MockTimes, StoredMock};
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

impl<T: Clone> Clone for MaybeFaux<T> {
    fn clone(&self) -> Self {
        match self {
            MaybeFaux::Real(r) => MaybeFaux::Real(r.clone()),
            MaybeFaux::Faux(_) => panic!("cannot clone a mock"),
        }
    }
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
    mocks: HashMap<usize, StoredMock>,
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
    pub unsafe fn unsafe_mock_once<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnOnce(I) -> O + Send,
    ) {
        self.mocks.insert(id as usize, StoredMock::once(mock));
    }

    pub fn mock_once<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnOnce(I) -> O + 'static + Send,
    ) where
        O: 'static,
    {
        unsafe {
            self.mocks.insert(id as usize, StoredMock::once(mock));
        }
    }

    /// # Safety
    ///
    /// Do not use this function without going through [When].  It is
    /// the caller's responsibility to pass a mock that is safe.
    ///
    /// [When]: When
    pub unsafe fn unsafe_mock<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnMut(I) -> O + Send,
        times: MockTimes,
    ) {
        self.mocks
            .insert(id as usize, StoredMock::many(mock, times));
    }

    pub fn mock<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnMut(I) -> O + 'static + Send,
        times: MockTimes,
    ) where
        O: 'static,
    {
        unsafe {
            self.mocks
                .insert(id as usize, StoredMock::many(mock, times));
        }
    }

    pub fn call_mock<R, I, O>(&mut self, id: fn(R, I) -> O, input: I) -> Option<O> {
        match self.mocks.entry(id as usize) {
            // no mock stored
            collections::hash_map::Entry::Vacant(_) => None,
            collections::hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                // a zero-times mock sneaked in here - delete
                StoredMock::Many(_, MockTimes::Times(0)) => {
                    entry.remove();
                    None
                }
                // only a single mock
                // remove and call the mock
                StoredMock::Once(_) | StoredMock::Many(_, MockTimes::Times(1)) => {
                    let mock = entry.remove();
                    match mock {
                        StoredMock::Once(mock) => {
                            let mock: Box<dyn FnOnce(I) -> O + Send> =
                                unsafe { std::mem::transmute(mock) };
                            Some(mock(input))
                        }
                        StoredMock::Many(mock, _) => {
                            let mut mock: Box<dyn FnMut(I) -> O + Send> =
                                unsafe { std::mem::transmute(mock) };
                            Some(mock(input))
                        }
                    }
                }
                // mock that can be called multiple times
                // call the mock but do not remove it
                StoredMock::Many(mock, times) => {
                    times.decrement();
                    let mock = unsafe {
                        &mut *(mock as *mut Box<dyn FnMut(()) + Send>
                            as *mut Box<dyn FnMut(I) -> O + Send>)
                    };
                    Some(mock(input))
                }
            },
        }
    }
}
