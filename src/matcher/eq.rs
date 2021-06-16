use std::{
    borrow::Borrow,
    fmt::{self, Formatter},
};

use super::ArgMatcher;

struct Eq<Expected>(Expected);

impl<Arg, Expected> ArgMatcher<Arg> for Eq<Expected>
where
    Arg: Borrow<Expected>,
    Expected: fmt::Debug + PartialEq,
{
    fn matches(&self, actual: &Arg) -> bool {
        &self.0 == actual.borrow()
    }
}

impl<Expected> fmt::Display for Eq<Expected>
where
    Expected: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Returns an equality matcher.
///
/// Returns a matcher that compares `Arg` using equality across
/// borrows.
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
/// // `Rc<T>` implements `Borrow<T>`
/// assert!(matcher::eq(5).matches(&Rc::new(5)));
///
/// // `&T` implements `Borrow<T>`
/// let ref_of_ref: &&i32 = &&5;
/// assert!(matcher::eq(5).matches(ref_of_ref));
/// ```
///
/// ## Usage within when!
///
/// For convenience, [`faux::when!`](crate::when!) defaults to the
/// `eq` matcher. See the [matcher
/// syntax](../macro.when.html#argument-matchers) for more
/// information.
///
/// ```ignore
/// // `_` means the `any` matcher
/// faux::when!(my_struct.some_method(2)).then_return(5);
///
/// // we can also call it manually within `when!`
/// faux::when!(my_struct.some_method(_ = faux::matcher::eq(2)))
///     .then_return(5);
///
/// // or call it manually outside `when!`
/// faux::when!(my_struct.some_method)
///     .with_args((matcher::eq(2),)).then_return(5);
/// ```
pub fn eq<Arg, Expected>(expected: Expected) -> impl ArgMatcher<Arg>
where
    Arg: Borrow<Expected>,
    Expected: fmt::Debug + PartialEq,
{
    Eq(expected)
}

struct EqAgainst<Expected>(Expected);

/// Returns an equality matcher for [different types].
///
/// Unlike [`eq`], it matches even if the types are different. This,
/// however, comes at the cost of not being able to match across
/// borrows.
///
/// # Examples
///
/// ```
/// use faux::matcher::{self, ArgMatcher};
///
/// // `String` implements `PartialEq<&str>`
/// assert!(matcher::eq_against("x".to_string()).matches(&"x"));
/// assert!(!matcher::eq_against("x".to_string()).matches(&"y"));
/// ```
///
/// ## Usage within when!
///
/// [`faux::when!`](crate::when!) does not have a special syntax for
/// this matcher. See the [matcher
/// syntax](../macro.when.html#argument-matchers) for more
/// information.
///
/// ```ignore
/// // we can call it within `when!`
/// faux::when!(my_struct.some_method(_ = faux::matcher::eq_against(2)))
///     .then_return(5);
///
/// // or call it outside `when!`
/// faux::when!(my_struct.some_method)
///     .with_args((matcher::eq_against(2),)).then_return(5);
/// ```
///
/// [different types]:
/// https://doc.rust-lang.org/std/cmp/trait.PartialEq.html#how-can-i-compare-two-different-types
pub fn eq_against<Arg>(expected: impl PartialEq<Arg> + fmt::Debug) -> impl ArgMatcher<Arg>
where
    Arg: ?Sized,
{
    EqAgainst(expected)
}

impl<Expected, Arg> ArgMatcher<Arg> for EqAgainst<Expected>
where
    Arg: ?Sized,
    Expected: fmt::Debug + PartialEq<Arg>,
{
    fn matches(&self, actual: &Arg) -> bool {
        &self.0 == actual
    }
}

impl<Expected> fmt::Display for EqAgainst<Expected>
where
    Expected: fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_ == {:?}", self.0)
    }
}
