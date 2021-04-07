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

/// Returns an universal argument matcher.
///
/// The returned matcher will match any `T`
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
pub fn any<T: ?Sized>() -> impl ArgMatcher<T> {
    Any
}
