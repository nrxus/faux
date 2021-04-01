use crate::mock::{Mock, SavedMock};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

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
    Faux(MockStore),
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
        MaybeFaux::Faux(MockStore::new())
    }
}

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: Mutex<HashMap<usize, Arc<Mutex<SavedMock<'static>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn mock<R, I, O>(&mut self, id: fn(R, I) -> O, mock: Mock<'static, I, O>) {
        self.store_mock(id, mock)
    }

    pub(crate) unsafe fn mock_unchecked<'a, R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: Mock<'a, I, O>,
    ) {
        // pretend the lifetime is static
        self.store_mock(id, std::mem::transmute(mock))
    }

    fn store_mock<R, I, O>(&mut self, id: fn(R, I) -> O, mock: Mock<'static, I, O>) {
        self.mocks.lock().unwrap().insert(
            id as usize,
            Arc::new(Mutex::new(unsafe { mock.unchecked() })),
        );
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_mock<R, I, O>(&self, id: fn(R, I) -> O, input: I) -> Result<O, String> {
        let locked_store = self.mocks.lock().unwrap();
        let stub = locked_store
            .get(&(id as usize))
            .cloned()
            .ok_or_else(|| "method was never mocked".to_string())?;

        // drop the lock before calling the mock to avoid deadlocking in the mock
        std::mem::drop(locked_store);

        let mut locked_mock = stub.lock().unwrap();
        locked_mock.call(input)
    }
}
