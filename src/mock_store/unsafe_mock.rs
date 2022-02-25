//! THIS IS ONE OF THE MOST DANGEROUS PARTS OF `FAUX`. PROCEED WITH
//! EXTREME CAUTION.

use std::fmt::{self, Formatter};

use super::mock::Mock;

/// Stores the a mock with its generics "erased"
///
/// This allows different mocks to be saved in the same collections.
/// Ideally we would use something like `std::any::Any` instead but
/// dynamic casting only works on static types and we do not want to
/// limit `faux` to only working with static inputs/outputs.
pub struct UnsafeMock<'a> {
    unsafe_mock: Mock<'a, (), (), 0>,
}

impl<'a> UnsafeMock<'a> {
    /// Returns a mutable reference to the mock with its types
    /// re-added.
    ///
    /// # Safety
    ///
    /// This method is *extremely* unsafe. This is only safe if you
    /// know precisely what the input (I), output, (O) and number of
    /// arguments (N) were of the original `Mock` this came from.
    pub unsafe fn as_checked_mut<I, O, const N: usize>(&mut self) -> &mut Mock<'a, I, O, N> {
        // Might be safer to only transmute only the matcher and stub
        // of each mock instead of the entire object. This works
        // though, and I don't see any reason why it wouldn't but if
        // we start seeing seg-faults this is a potential thing to
        // change.
        let mock = &mut self.unsafe_mock;
        std::mem::transmute(mock)
    }
}

impl fmt::Debug for UnsafeMock<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // do not try to access the type-erased stub or matcher
        f.debug_struct("UnsafeMock")
            .field("stubs_len", &self.unsafe_mock.stubs.len())
            .finish()
    }
}

impl<'a, I, O, const N: usize> From<Mock<'a, I, O, N>> for UnsafeMock<'a> {
    fn from(stub: Mock<'a, I, O, N>) -> Self {
        // Safety:
        // The only posible actions on the returned `Saved` are:
        // * as_checked_mut: already marked as `unsafe`
        // * debug format: does not look into the unsafe fields
        unsafe {
            let unsafe_mock = std::mem::transmute(stub);
            Self { unsafe_mock }
        }
    }
}
