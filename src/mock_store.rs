mod mock;
mod unsafe_mock;

pub mod stub;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub use stub::Stub;

use self::{mock::Mock, unsafe_mock::UnsafeMock};

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    mocks: Arc<Mutex<HashMap<usize, Arc<Mutex<UnsafeMock<'static>>>>>>,
}

impl MockStore {
    pub fn new() -> Self {
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
        let mock = {
            let locked_store = self.mocks.lock().unwrap();
            locked_store
                .get(&(id as usize))
                // clone so we can drop locked_store and not block
                // other call_stubs
                .cloned()
                .ok_or_else(|| "✗ method was never stubbed".to_string())
        }?;

        let mut mock = mock.lock().unwrap();
        let mock: &mut Mock<I, O, N> = mock.as_checked_mut();

        mock.call(input).map_err(|errors| {
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
        // we could clone here to release the store immediately but
        // adding a stub to a mocked method is quick and not dependent
        // on user code so cloning the `Arc<_>` is unnecessary
        let mut locked_store = self.mocks.lock().unwrap();

        use std::collections::hash_map::Entry;
        match locked_store.entry(id as usize) {
            Entry::Occupied(o) => {
                let mut method = o.into_mut().lock().unwrap();
                let method = unsafe { method.as_checked_mut() };
                method.add_stub(stub);
            }
            Entry::Vacant(v) => {
                let mut method = Mock::new();
                method.add_stub(stub);
                v.insert(Arc::new(Mutex::new(method.into())));
            }
        };
    }
}
