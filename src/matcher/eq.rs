use std::{
    borrow::Borrow,
    fmt::{self, Formatter},
};

use super::ArgMatcher;

/// Equality matcher for equal types
///
/// Unlike [EqAgainst], if only allows equality matching of the same
/// type. This comes at the gained benefit of being able to match
/// across borrows. This means that Eq<T> implements not only
/// `ArgMatcher<T>`, but also `ArgMatcher<&T>`, and more generally
/// `ArgMatcher<Borrow<T>>`
pub struct Eq<Expected>(Expected);

impl<Arg: Borrow<Expected>, Expected: fmt::Debug + PartialEq> ArgMatcher<Arg> for Eq<Expected> {
    fn matches(&self, actual: &Arg) -> bool {
        &self.0 == actual.borrow()
    }
}

impl<Expected: fmt::Debug> fmt::Display for Eq<Expected> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Creates an [`Eq`](struct@Eq) matcher.
pub fn eq<Expected: fmt::Debug + PartialEq>(expected: Expected) -> Eq<Expected> {
    Eq(expected)
}

/// Equality matcher for different types
///
/// Unlike [`Eq`](struct@Eq), it matches even if the types are
/// different. This, however, comes at the cost of not being able to
/// match across borrows.
pub struct EqAgainst<Expected>(Expected);

/// Creates an [`EqAgainst`] matcher.
pub fn eq_against<Expected>(expected: Expected) -> EqAgainst<Expected> {
    EqAgainst(expected)
}

impl<Expected: fmt::Debug + PartialEq<Arg>, Arg> ArgMatcher<Arg> for EqAgainst<Expected> {
    fn matches(&self, actual: &Arg) -> bool {
        &self.0 == actual
    }
}

impl<Expected: fmt::Debug> fmt::Display for EqAgainst<Expected> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_ == {:?}", self.0)
    }
}
