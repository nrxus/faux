//! THIS IS ONE OF THE MOST DANGEROUS PARTS OF `FAUX`. PROCEED WITH
//! EXTREME CAUTION.

use std::fmt::{self, Formatter};

use super::Mock;

/// Stores the a mock with its generics "erased"
///
/// This allows different mocks to be saved in the same collections.
/// Ideally we would use something like `std::any::Any` instead but
/// dynamic casting only works on static types and we do not want to
/// limit `faux` to only working with static inputs/outputs.
pub struct Unchecked<'stub> {
    unsafe_mock: Mock<'stub, (), ()>,
    debug_repr: String,
}

impl<'stub> Unchecked<'stub> {
    /// Returns a reference to the mock with its types re-added.
    ///
    /// # Safety
    ///
    /// This method is *extremely* unsafe. This is only safe if you
    /// know precisely what the input (I), output (O) were of the
    /// original [`Mock`] this came from.
    pub unsafe fn as_typed<I, O>(&self) -> &Mock<'stub, I, O> {
        // Might be safer to only transmute only the matcher and stub
        // of each mock instead of the entire object. This works
        // though, and I don't see any reason why it wouldn't but if
        // we start seeing seg-faults this is a potential thing to
        // change.
        let mock = &self.unsafe_mock;
        std::mem::transmute(mock)
    }

    /// Returns a mutable reference to the mock with its types
    /// re-added.
    ///
    /// # Safety
    ///
    /// This method is *extremely* unsafe. This is only safe if you
    /// know precisely what the input (I), output (O) were of the
    /// original [`Mock`] this came from.
    pub unsafe fn as_typed_mut<I, O>(&mut self) -> &mut Mock<'stub, I, O> {
        // Might be safer to only transmute only the matcher and stub
        // of each mock instead of the entire object. This works
        // though, and I don't see any reason why it wouldn't but if
        // we start seeing seg-faults this is a potential thing to
        // change.
        let mock = &mut self.unsafe_mock;
        std::mem::transmute(mock)
    }
}

impl fmt::Debug for Unchecked<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.debug_repr)
    }
}

impl<'stub, I, O> From<Mock<'stub, I, O>> for Unchecked<'stub> {
    fn from(mock: Mock<I, O>) -> Self {
        // we are about to lose all information about the mock so
        // let's save its debug representation so we can print it as
        // our own
        let debug_repr = format!("{:?}", mock);
        // Safety:
        // The only posible actions on the returned `Saved` are:
        // * as_checked_mut: already marked as `unsafe`
        // * debug format: does not look into the unsafe fields
        unsafe {
            let unsafe_mock = std::mem::transmute(mock);
            Self {
                unsafe_mock,
                debug_repr,
            }
        }
    }
}
