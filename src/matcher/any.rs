use crate::ArgMatcher;

pub struct Any;

impl<T> ArgMatcher<T> for Any {
    type Message = &'static str;

    fn matches(&self, _: &T) -> bool {
        true
    }

    fn message(&self) -> Self::Message {
        "_"
    }
}

pub fn any() -> Any {
    Any
}
