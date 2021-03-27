//! Tools to check if an argument to a mocked method matches a
//! determined expectation.

mod all_args;
mod any;
mod eq;

pub use all_args::AllArgs;
pub use any::{any, Any};
pub use eq::{eq, eq_against, Eq, EqAgainst};

use std::fmt::{self, Formatter};

/// Trait to check if an argument to a mocked method matches a
/// determined expectation.
pub trait ArgMatcher<Arg>: fmt::Display {
    /// Checks if the argument matches some determined expection.
    ///
    /// ```
    /// use faux::matcher::{self, ArgMatcher};
    ///
    /// let eq_five = matcher::eq(5);
    /// assert!(eq_five.matches(&5));
    /// assert!(!eq_five.matches(&4));
    /// ```
    fn matches(&self, argument: &Arg) -> bool;

    /// Converts the `Argmatcher<Arg>` into an `ArgMatcher<&Arg>` to
    /// test against the reference of the argument.
    fn ref_of(self) -> RefOf<Self>
    where
        Self: Sized,
    {
        RefOf(self)
    }
}

/// Wraps an `Argmatcher<Arg>` and implements `ArgMatcher<&Arg>`
/// instead
pub struct RefOf<AM>(AM);

impl<Arg, AM: ArgMatcher<Arg>> ArgMatcher<&Arg> for RefOf<AM> {
    fn matches(&self, actual: &&Arg) -> bool {
        self.0.matches(*actual)
    }
}

impl<AM: fmt::Display> fmt::Display for RefOf<AM> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "&{}", self.0)
    }
}
