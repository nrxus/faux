use super::{ArgMatcher, InvocationMatcher};
use std::fmt::{self, Formatter};

/// Universal matcher.
pub struct Any;

impl<T> ArgMatcher<T> for Any {
    /// Always returns true
    fn matches(&self, _: &T) -> bool {
        true
    }
}

impl<Arg> InvocationMatcher<Arg> for Any {
    /// Always returns Ok(())
    fn matches(&self, _: &Arg) -> Result<(), String> {
        Ok(())
    }
}

impl fmt::Display for Any {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}

/// Creates an [`Any`] matcher.
pub fn any() -> Any {
    Any
}
