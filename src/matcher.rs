use std::fmt::{Debug, Formatter, Result};

pub trait ArgMatcher<T> {
    fn matches(&self, arg: T) -> bool;
}

impl<T: Debug> Debug for Ref<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "&{:?}", self.0)
    }
}

pub struct Ref<T>(pub T);

pub struct EqMatcher<T>(T);

impl<T: Debug> Debug for EqMatcher<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}

pub struct AnyMatcher;

impl Debug for AnyMatcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "_")
    }
}

impl<T, A> PartialEq<A> for Ref<T>
where
    for<'a> A: PartialEq<&'a T>,
{
    fn eq(&self, rhs: &A) -> bool {
        *rhs == &self.0
    }
}

impl<T> ArgMatcher<T> for AnyMatcher {
    fn matches(&self, _: T) -> bool {
        true
    }
}

impl<Arg, Captured: PartialEq<Arg>> ArgMatcher<Arg> for EqMatcher<Captured> {
    fn matches(&self, arg: Arg) -> bool {
        self.0 == arg
    }
}

pub fn eq<T>(arg: T) -> EqMatcher<T> {
    EqMatcher(arg)
}

pub fn any() -> AnyMatcher {
    AnyMatcher
}

impl ArgMatcher<()> for () {
    fn matches(&self, _: ()) -> bool {
        true
    }
}

impl<A, B, AM: ArgMatcher<A>, BM: ArgMatcher<B>> ArgMatcher<(A, B)> for (AM, BM) {
    fn matches(&self, (a, b): (A, B)) -> bool {
        self.0.matches(a) && self.1.matches(b)
    }
}

impl<A, B, C, AM: ArgMatcher<A>, BM: ArgMatcher<B>, CM: ArgMatcher<C>> ArgMatcher<(A, B, C)>
    for (AM, BM, CM)
{
    fn matches(&self, (a, b, c): (A, B, C)) -> bool {
        self.0.matches(a) && self.1.matches(b) && self.2.matches(c)
    }
}
