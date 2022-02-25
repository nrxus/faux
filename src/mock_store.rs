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

#[derive(Debug, Default)]
struct MockedMethod {
    stubs: Vec<stub::Saved<'static>>,
}

impl MockedMethod {
    pub fn add_stub<I, O, const N: usize>(&mut self, stub: Stub<'static, I, O, N>) {
        self.stubs.push(unsafe { stub.unchecked() })
    }

    pub unsafe fn call<I, O, const N: usize>(&mut self, mut input: I) -> Result<O, Vec<String>> {
        let mut errors = vec![];

        for stub in self.stubs.iter_mut().rev() {
            match stub.call::<_, _, N>(input) {
                Err((i, e)) => {
                    errors.push(format!("✗ {}", e));
                    input = i
                }
                Ok(o) => return Ok(o),
            }
        }

        Err(errors)
    }
}

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: Arc<Mutex<HashMap<usize, Arc<Mutex<MockedMethod>>>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore::default()
    }

    // could be &self since it's all under a Mutex<_> but conceptually
    // we are adding a new stub which is a mutation
    pub(crate) fn stub<R, I, O: 'static, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'static, I, O, N>,
    ) {
        self.store_stub(id, stub)
    }

    // could be &self since it's all under a Mutex<_> but conceptually
    // we are adding a new stub which is a mutation
    pub(crate) unsafe fn stub_unchecked<R, I, O, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'_, I, O, N>,
    ) {
        // pretend the lifetime is static
        self.store_stub::<_, _, _, N>(id, std::mem::transmute(stub))
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_stub<R, I, O, const N: usize>(
        &self,
        id: fn(R, I) -> O,
        input: I,
    ) -> Result<O, String> {
        let locked_store = self.mocks.lock().unwrap();
        let potential_mocks = locked_store
            .get(&(id as usize))
            .cloned() // clone so we can unlock the mock_store immediately
            .ok_or_else(|| "✗ method was never stubbed".to_string())?;

        // drop the lock before calling the stub to avoid deadlocking in the mock
        std::mem::drop(locked_store);

        let mut locked_mocks = potential_mocks.lock().unwrap();

        locked_mocks.call::<_, _, N>(input).map_err(|errors| {
            if errors.is_empty() {
                "✗ method was never stubbed".to_string()
            } else {
                errors.join("\n\n")
            }
        })
    }

    // could be &self since it's all under a Mutex<_> but conceptually
    // we are adding a new stub which is a mutation
    fn store_stub<R, I, O, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
        stub: Stub<'static, I, O, N>,
    ) {
        let mut locked_store = self.mocks.lock().unwrap();
        // we could clone here to release the store immediately but
        // adding a stub to a mocked method is quick and not dependent
        // on user code so cloning the `Arc<_>` is unnecessary
        let method = locked_store.entry(id as usize).or_default();
        method.lock().unwrap().add_stub(stub);
    }
}
