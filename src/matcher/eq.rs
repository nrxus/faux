use std::{
    borrow::Borrow,
    fmt::{self, Formatter},
};

use super::ArgMatcher;

/// Equality matcher.
///
/// Unlike [`EqAgainst`], it only allows equality matching of the same
/// type. This comes at the benefit of being able to match across
/// borrows. This means `Eq<T>` implements not only `ArgMatcher<T>`, but
/// also `ArgMatcher<&T>`. More generally, `Eq<T>` implements `ArgMatcher<Borrow<T>>`
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

/// Equality matcher for [different types].
///
/// Unlike [`Eq`](struct@Eq), it matches even if the types are
/// different. This, however, comes at the cost of not being able to
/// match across borrows.
///
/// [different types]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#how-can-i-compare-two-different-types
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
