use super::ArgMatcher;
use std::fmt::{self, Formatter};

struct Any;

impl<T> ArgMatcher<T> for Any {
    fn matches(&self, _: &T) -> bool {
        true
    }
}

impl fmt::Display for Any {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

/// Universal argument matcher.
///
/// The returned [`ArgMatcher<T>`](ArgMatcher) will match any `T`
pub fn any<T>() -> impl ArgMatcher<T> {
    Any
}
