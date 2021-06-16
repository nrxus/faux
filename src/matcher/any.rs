use super::ArgMatcher;
use std::fmt::{self, Formatter};

struct Any;

impl<T: ?Sized> ArgMatcher<T> for Any {
    fn matches(&self, _: &T) -> bool {
        true
    }
}

impl fmt::Display for Any {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

/// Returns a universal argument matcher.
///
/// The returned matcher will match any `T`
///
///
/// # Examples
///
/// ```
/// struct Data;
///
/// use faux::matcher::{self, ArgMatcher};
///
/// assert!(matcher::any().matches(&5));
/// assert!(matcher::any().matches("hello"));
/// assert!(matcher::any().matches(&Data));
/// ```
///
/// ## Usage within when!
///
/// For convenience, [`faux::when!`](crate::when!) uses `_` to denote
/// the `any` matcher. See the [matcher
/// syntax](../macro.when.html#argument-matchers) for more
/// information.
///
/// ```ignore
/// // `_` means the `any` matcher
/// faux::when!(my_struct.some_method(_)).then_return(5);
///
/// // we can also call it manually within `when!`
/// faux::when!(my_struct.some_method(_ = faux::matcher::any()))
///     .then_return(5);
///
/// // or call it manually outside `when!`
/// faux::when!(my_struct.some_method)
///     .with_args((matcher::any(),)).then_return(5);
/// ```
pub fn any<T: ?Sized>() -> impl ArgMatcher<T> {
    Any
}
