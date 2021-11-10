use crate::stub::{self, Stub};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

#[derive(Clone, Debug)]
pub enum MaybeFaux<T> {
    Real(T),
    Faux(MockStore),
}

impl<T: Default> Default for MaybeFaux<T> {
    fn default() -> Self {
        MaybeFaux::Real(T::default())
    }
}

impl<T> MaybeFaux<T> {
    pub fn faux() -> Self {
        MaybeFaux::Faux(MockStore::new())
    }
}

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    stubs: Arc<Mutex<HashMap<usize, Arc<Mutex<Vec<stub::Saved<'static>>>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            stubs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) fn stub<R, I, O: 'static, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'static, I, O, N>,
    ) {
        self.store_stub::<_, _, _, N>(id, stub)
    }

    pub(crate) unsafe fn stub_unchecked<R, I, O, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'_, I, O, N>,
    ) {
        // pretend the lifetime is static
        self.store_stub::<_, _, _, N>(id, std::mem::transmute(stub))
    }

    fn store_stub<R, I, O, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'static, I, O, N>,
    ) {
        let stubs = self
            .stubs
            .lock()
            .unwrap()
            .entry(id as usize)
            .or_default()
            .clone();

        stubs.lock().unwrap().push(unsafe { stub.unchecked() });
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_stub<R, I, O, const N: usize>(
        &self,
        id: fn(R, I) -> O,
        mut input: I,
    ) -> Result<O, String> {
        let locked_store = self.stubs.lock().unwrap();
        let potential_stubs = locked_store
            .get(&(id as usize))
            .cloned()
            .ok_or_else(|| "✗ method was never stubbed".to_string())?;

        // drop the lock before calling the stub to avoid deadlocking in the mock
        std::mem::drop(locked_store);

        let mut potential_subs = potential_stubs.lock().unwrap();
        let mut errors = vec![];

        for stub in potential_subs.iter_mut().rev() {
            match stub.call::<_, _, N>(input) {
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
