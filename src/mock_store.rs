use crate::mock::{Mock, SavedMock};
use std::collections::HashMap;
use std::sync::{atomic, Arc, Mutex};

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
    mocks: Mutex<HashMap<u64, Arc<Mutex<Vec<SavedMock<'static>>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            mocks: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn mock<I, O: 'static>(&mut self, id: u64, mock: Mock<'static, I, O>) {
        self.store_mock(id, mock)
    }

    pub(crate) unsafe fn mock_unchecked<'a, I, O>(&mut self, id: u64, mock: Mock<'a, I, O>) {
        // pretend the lifetime is static
        self.store_mock::<I, O>(id, std::mem::transmute(mock))
    }

    fn store_mock<I, O>(&mut self, id: u64, mock: Mock<'static, I, O>) {
        let mocks = self.mocks.lock().unwrap().entry(id).or_default().clone();

        mocks.lock().unwrap().push(unsafe { mock.unchecked() });
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_mock<I, O>(&self, id: u64, mut input: I) -> Result<O, String> {
        let locked_store = self.mocks.lock().unwrap();
        let potential_mocks = locked_store
            .get(&id)
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

#[doc(hidden)]
/// Similar to `once_cell::race::OnceNonZeroUsize`.
///
/// This need only be unique within a given mocked type, so our current strategy of
/// basically generating a LazyMethodId per-mocked-method should be fine.
pub struct LazyMethodId(atomic::AtomicU64);

impl LazyMethodId {
    pub const fn new() -> Self {
        LazyMethodId(atomic::AtomicU64::new(0))
    }

    pub fn get(&self) -> u64 {
        let id = self.0.load(atomic::Ordering::Acquire);
        if id != 0 {
            return id;
        }
        let id = GLOBAL_ID_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        let res =
            self.0
                .compare_exchange(0, id, atomic::Ordering::AcqRel, atomic::Ordering::Acquire);
        match res {
            Ok(_) => id,
            // another thread was initializing at the same time and it won the race
            Err(winner_id) => winner_id,
        }
    }
}

static GLOBAL_ID_COUNTER: atomic::AtomicU64 = atomic::AtomicU64::new(1);
