//! Tools to check if an argument to a mocked method matches
//! expectations.

mod any;
mod from_fn;
mod eq;
mod invocation_matcher;

pub use any::{any, Any};
pub use eq::{eq, eq_against, Eq, EqAgainst};
pub use from_fn::from_fn;
pub use invocation_matcher::InvocationMatcher;

use std::fmt::{self, Formatter};

/// Matcher for single argument of a method.
///
/// Implementors provide an expectation to match an argument against.
///
/// `faux` provides some simple matchers, such as [`Eq`](struct@Eq)
/// for equality. Check [Implementors](#implementors) for the
/// exhaustive list.
///
/// You may define your own matcher for special use cases. The
/// [`fmt::Display`] implementation is used by [`InvocationMatcher`]
/// to display the expectation when any arguments failed to match.
///
/// # Examples
///
/// ```
/// use std::fmt::{self, Formatter};
/// use faux::ArgMatcher;
///
/// struct HasLength(usize);
///
/// // displayed as the expectation when any argument fails to match
/// // when used by an `InvocationMatcher`
/// impl fmt::Display for HasLength {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
///         write!(f, "_.len() == {}", self.0)
///     }
/// }
///
/// impl <T> ArgMatcher<&[T]> for HasLength {
///     // matches takes a reference to the argument
///     // the argument is &[T] so it takes &&[T]
///     fn matches(&self, argument: &&[T]) -> bool {
///         argument.len() == self.0
///     }
/// }
///
/// let has_three_length = HasLength(3);
/// let vec = vec![56, 78, 12, 43, 23];
/// assert!(has_three_length.matches(&&vec[..3]));
/// assert!(!has_three_length.matches(&&vec[1..]));
///
/// ```
pub trait ArgMatcher<Arg>: fmt::Display {
    /// Checks if the argument matches the determined expectation.
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
    fn into_ref_matcher(self) -> RefMatcher<Self>
    where
        Self: Sized,
    {
        RefMatcher(self)
    }
}

/// Wraps an `ArgMatcher<Arg>` and implements `ArgMatcher<&Arg>`
/// instead.
pub struct RefMatcher<AM>(AM);

impl<Arg, AM: ArgMatcher<Arg>> ArgMatcher<&Arg> for RefMatcher<AM> {
    fn matches(&self, actual: &&Arg) -> bool {
        self.0.matches(*actual)
    }
}

impl<AM: fmt::Display> fmt::Display for RefMatcher<AM> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "*{}", self.0)
    }
}
