use crate::mock::{Mock, SavedMock};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
///
/// ```
/// fn implements_default<T: Default>(_: T) {}
///
/// implements_default(3);
/// implements_default(faux::MaybeFaux::Real(3));
/// ```

#[derive(Debug)]
pub enum MaybeFaux<T> {
    Real(T),
    Faux(MockStore),
}

impl<T: Default> Default for MaybeFaux<T> {
    fn default() -> Self {
        MaybeFaux::Real(T::default())
    }
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
    mocks: Mutex<HashMap<usize, Arc<Mutex<Vec<SavedMock<'static>>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn mock<R, I, O: 'static>(&mut self, id: fn(R, I) -> O, mock: Mock<'static, I, O>) {
        self.store_mock(id, mock)
    }

    pub(crate) unsafe fn mock_unchecked<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        mock: Mock<'_, I, O>,
    ) {
        // pretend the lifetime is static
        self.store_mock(id, std::mem::transmute(mock))
    }

    fn store_mock<R, I, O>(&mut self, id: fn(R, I) -> O, mock: Mock<'static, I, O>) {
        let mocks = self
            .mocks
            .lock()
            .unwrap()
            .entry(id as usize)
            .or_default()
            .clone();

        mocks.lock().unwrap().push(unsafe { mock.unchecked() });
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_mock<R, I, O>(&self, id: fn(R, I) -> O, mut input: I) -> Result<O, String> {
        let locked_store = self.mocks.lock().unwrap();
        let potential_mocks = locked_store
            .get(&(id as usize))
            .cloned()
            .ok_or_else(|| "✗ method was never mocked".to_string())?;

        // drop the lock before calling the mock to avoid deadlocking in the mock
        std::mem::drop(locked_store);

        let mut potential_mocks = potential_mocks.lock().unwrap();
        let mut errors = vec![];

        for mock in potential_mocks.iter_mut().rev() {
            match mock.call(input) {
                Err((i, e)) => {
                    errors.push(format!("✗ {}", e));
                    input = i
                }
                Ok(o) => return Ok(o),
            }
        }

        assert!(!errors.is_empty());

        Err(errors.join("\n\n"))
    }
}
