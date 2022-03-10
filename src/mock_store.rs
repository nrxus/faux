mod mock;
mod unsafe_mock;

pub mod stub;

use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

pub use stub::Stub;

use self::{mock::Mock, unsafe_mock::UnsafeMock};

#[derive(Clone, Debug, Default)]
#[doc(hidden)]
pub struct SharedMockStore {
    inner: Arc<RwLock<MockStore>>,
}

impl SharedMockStore {
    pub fn new() -> Self {
        SharedMockStore::default()
    }

    pub(crate) fn get_unique<R, I, O, const N: usize>(
        &mut self,
        id: fn(R, I) -> O,
    ) -> Option<&mut Mock<'static, I, O, N>> {
        let mocks = Arc::get_mut(&mut self.inner)?;

        let mock = mocks.get_mut().mocks.entry(id as usize).or_insert_with(|| {
            let method: Mock<I, O, N> = Mock::new();
            method.into()
        });

        // Safety: The mock was inserted by us so we know the
        // conversion back into a checked mock is safe
        Some(unsafe { mock.as_typed_mut() })
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
        let mock = self.inner.read();
        let mock: &Mock<_, _, N> = mock
            .get_callable(id)
            .ok_or_else(|| "✗ method was never stubbed".to_string())?;

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
        let mocks = self.get_unique(id).expect("faux: mock is not unique");
        mocks.add_stub(stub);
    }
}

#[derive(Debug, Default)]
struct MockStore {
    mocks: HashMap<usize, UnsafeMock<'static>>,
}

impl MockStore {
    pub unsafe fn get_callable<R, I, O, const N: usize>(
        &self,
        id: fn(R, I) -> O,
    ) -> Option<&Mock<'static, I, O, N>> {
        self.mocks.get(&(id as usize)).map(|m| m.as_typed())
    }
}
