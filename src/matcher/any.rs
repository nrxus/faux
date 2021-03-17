use super::ArgMatcher;
use std::fmt::{self, Formatter};

pub struct Any;

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

pub fn any() -> Any {
    Any
}
