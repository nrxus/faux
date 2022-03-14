pub mod stub;

mod store;
mod unchecked;

use std::fmt::{self, Formatter};

pub use self::{store::Store, stub::Stub};

use parking_lot::Mutex;

/// A function mock
///
/// Stores information about a mock, such as its stubs, with its
/// inputs and output typed.
pub struct Mock<'stub, I, O> {
    pub(super) stubs: Vec<Mutex<Stub<'stub, I, O>>>,
}

impl<'stub, I, O> Mock<'stub, I, O> {
    /// Creates an empty mock
    pub fn new() -> Self {
        Self { stubs: vec![] }
    }

    /// Attempts to invoke the mock
    ///
    /// Checks the given input against the stored stubs, invoking the
    /// first stub whose invocation matcher suceeds for the
    /// inputs. The stubs are checked in reverse insertion order such
    /// that the last inserted stub is the first attempted
    /// one. Returns an error if no stub is found for the given input.
    pub fn call(&self, mut input: I) -> Result<O, Vec<String>> {
        let mut errors = vec![];

        for stub in self.stubs.iter().rev() {
            match stub.lock().call(input) {
                Err((i, e)) => {
                    errors.push(format!("✗ {}", e));
                    input = i
                }
                Ok(o) => return Ok(o),
            }
        }

        Err(errors)
    }

    /// Adds a new stub for the mocked function
    pub fn add_stub(&mut self, stub: Stub<'stub, I, O>) {
        self.stubs.push(Mutex::new(stub))
    }
}

impl<I, O> Default for Mock<'_, I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> fmt::Debug for Mock<'_, I, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mock").field("stubs", &self.stubs).finish()
    }
}
