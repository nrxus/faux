use std::{
    borrow::Borrow,
    fmt::{self, Formatter},
};

use super::ArgMatcher;

struct Eq<Expected>(Expected);

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

/// Equality matcher.
///
/// Returns an [`ArgMatcher<Arg>`](ArgMatcher) that compares `Arg`
/// using equality across borrows.
///
/// Unlike [`eq_against`], it only allows equality matching for the
/// same type. This comes at the benefit of being able to match across
/// borrows.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use faux::matcher::{self, ArgMatcher};
///
/// let eq_four = matcher::eq(4);
/// assert!(eq_four.matches(&4));
/// assert!(!eq_four.matches(&5));
/// ```
///
/// ## Matching across borrows
///
/// ```
/// use std::rc::Rc;
/// use faux::matcher::{self, ArgMatcher};
///
/// // Rc<T> implements Borrow<T>
/// assert!(matcher::eq(5).matches(&Rc::new(5)));
///
/// // &T implements Borrow<T>
/// let ref_of_ref: &&i32 = &&5;
/// assert!(matcher::eq(5).matches(ref_of_ref));
/// ```
pub fn eq<Arg: Borrow<Expected>, Expected: fmt::Debug + PartialEq>(
    expected: Expected,
) -> impl ArgMatcher<Arg> {
    Eq(expected)
}

struct EqAgainst<Expected>(Expected);
/// Equality matcher for [different types].
///
/// Returns an [`ArgMatcher<Arg`](ArgMatcher) that compares `Arg`
/// using equality against a different type.
///
/// Unlike [`eq`], it matches even if the types are different. This,
/// however, comes at the cost of not being able to match across
/// borrows.
///
/// [different types]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#how-can-i-compare-two-different-types
pub fn eq_against<Arg>(expected: impl PartialEq<Arg> + fmt::Debug) -> impl ArgMatcher<Arg> {
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
