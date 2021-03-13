use crate::mock::{MockTimes, StoredMock, UncheckedMock};
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

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: HashMap<usize, UncheckedMock<'static>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: HashMap::new(),
        }
    }

    pub(crate) unsafe fn mock_once_unchecked<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnOnce(I) -> O + Send,
    ) {
        // pretend the lifetime is static
        self.store_mock(id, std::mem::transmute(StoredMock::once(mock)))
    }

    pub(crate) fn mock_once<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnOnce(I) -> O + 'static + Send,
    ) {
        self.store_mock(id, StoredMock::once(mock));
    }

    pub(crate) unsafe fn mock_unchecked<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnMut(I) -> O + Send,
        times: MockTimes,
    ) {
        // pretend the lifetime is static
        self.store_mock(id, std::mem::transmute(StoredMock::many(mock, times)))
    }

    pub(crate) fn mock<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: impl FnMut(I) -> O + 'static + Send,
        times: MockTimes,
    ) {
        self.store_mock(id, StoredMock::many(mock, times))
    }

    fn store_mock<R, I, O>(&mut self, id: fn(R, I) -> O, mock: StoredMock<'static, I, O>) {
        self.mocks
            .insert(id as usize, unsafe { UncheckedMock::new(mock) });
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_mock<R, I, O>(&mut self, id: fn(R, I) -> O, input: I) -> Option<O> {
        match self.mocks.entry(id as usize) {
            // no mock stored
            collections::hash_map::Entry::Vacant(_) => None,
            collections::hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
                // a zero-times mock sneaked in here - delete
                UncheckedMock::Many(_, MockTimes::Times(0)) => {
                    entry.remove();
                    None
                }
                // only a single mock
                // remove and call the mock
                UncheckedMock::Once(_) | UncheckedMock::Many(_, MockTimes::Times(1)) => {
                    let mock = entry.remove().transmute::<I, O>();
                    match mock {
                        StoredMock::Once(mock) => Some(mock(input)),
                        StoredMock::Many(mut mock, _) => Some(mock(input)),
                    }
                }
                // mock that can be called multiple times
                // call the mock but do not remove it
                UncheckedMock::Many(mock, times) => {
                    times.decrement();
                    let mock = &mut *(mock as *mut Box<dyn FnMut(()) + Send>
                        as *mut Box<dyn FnMut(I) -> O + Send>);
                    Some(mock(input))
                }
            },
        }
    }
}
