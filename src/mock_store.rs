use crate::{
    matcher::InvocationMatcher,
    stub::{self, Stub},
};
use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{self, Formatter},
    hash::Hash,
    sync::{Arc, Mutex},
    thread,
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

#[derive(Debug)]
struct Expectations {
    method_name: &'static str,
}

#[derive(Debug, Eq)]
struct ExpectedFn {
    id: usize,
    name: &'static str,
}

impl Borrow<usize> for ExpectedFn {
    fn borrow(&self) -> &usize {
        &self.id
    }
}

impl PartialEq for ExpectedFn {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for ExpectedFn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

pub struct SavedExpectation {
    expectation: Vec<String>,
    transmuted_matcher: Box<dyn InvocationMatcher<()> + Send>,
}

impl<I> From<Box<dyn InvocationMatcher<I> + Send>> for SavedExpectation {
    fn from(matcher: Box<dyn InvocationMatcher<I> + Send>) -> Self {
        SavedExpectation {
            expectation: matcher.expectations(),
            transmuted_matcher: unsafe { std::mem::transmute(matcher) },
        }
    }
}

impl fmt::Debug for SavedExpectation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SavedExpectation")
            .field("expectation", &self.expectation)
            .finish()
    }
}

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct MockStore {
    stubs: Mutex<HashMap<usize, Arc<Mutex<Vec<stub::Saved<'static>>>>>>,
    expectations: Mutex<HashMap<ExpectedFn, Vec<SavedExpectation>>>,
}

impl MockStore {
    fn new() -> Self {
        MockStore {
            stubs: Mutex::new(HashMap::new()),
            expectations: Mutex::new(HashMap::new()),
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

    pub fn expect<R, I, O>(
        &mut self,
        id: fn(R, I) -> O,
        name: &'static str,
        matcher: Box<dyn InvocationMatcher<I> + Send>,
    ) -> &mut SavedExpectation {
        let expectations = self.expectations.get_mut().unwrap();
        let expectations = expectations
            .entry(ExpectedFn {
                id: id as usize,
                name,
            })
            .or_default();

        expectations.push(matcher.into());

        expectations.last_mut().unwrap()
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

        let mut expectations = self.expectations.lock().unwrap();
        if let Some(expectations) = expectations.get_mut(&(id as usize)) {
            expectations.retain(|e| {
                let matcher = &*(&e.transmuted_matcher as *const Box<_>
                    as *const Box<dyn InvocationMatcher<I>>);
                matcher.matches(&input).is_err()
            })
        }
        // drop the lock before calling the mock to avoid deadlocking in the mock
        std::mem::drop(expectations);

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

impl Drop for MockStore {
    fn drop(&mut self) {
        if thread::panicking() {
            return;
        }

        let expectations = self.expectations.get_mut().unwrap();
        let expectations: Vec<_> = expectations
            .drain()
            .flat_map(|(e, v)| {
                v.into_iter()
                    .map(move |v| format!("  {}({:?})", e.name, v.expectation))
            })
            .collect();

        if expectations.is_empty() {
            return;
        }

        let unfulfilled = expectations.join("\n");

        panic!(
            "failed when dropping mock:\n✗ Expected invocations were not matched:\n{}\n",
            unfulfilled
        );
    }
}
