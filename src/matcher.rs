mod all_args;
mod any;
mod eq;

pub use all_args::{AllArgs, Single};
pub use any::{any, Any};
pub use eq::{eq, Eq};
use std::fmt::{self, Formatter};

pub trait ArgMatcher<Arg> {
    type Message: fmt::Display;

    fn matches(&self, arg: &Arg) -> bool;
    fn message(&self) -> Self::Message;
}

impl<T: fmt::Debug> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "&{:?}", self.0)
    }
}

pub struct Ref<T>(pub T);

impl<T, A> PartialEq<A> for Ref<T>
where
    for<'a> A: PartialEq<&'a T>,
{
    fn eq(&self, rhs: &A) -> bool {
        *rhs == &self.0
    }
}
