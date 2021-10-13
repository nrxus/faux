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
    stubs: Mutex<HashMap<usize, Arc<Mutex<Vec<stub::Saved<'static>>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            stubs: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn stub<R, I, O: 'static>(&mut self, id: fn(R, I) -> O, stub: Stub<'static, I, O>) {
        self.store_stub(id, stub)
    }

    pub(crate) unsafe fn stub_unchecked<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'_, I, O>,
    ) {
        // pretend the lifetime is static
        self.store_stub(id, std::mem::transmute(stub))
    }

    fn store_stub<R, I, O>(&mut self, id: fn(R, I) -> O, stub: Stub<'static, I, O>) {
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
    pub unsafe fn call_stub<R, I, O>(&self, id: fn(R, I) -> O, mut input: I) -> Result<O, String> {
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
            match stub.call(input) {
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
