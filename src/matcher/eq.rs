use std::{
    borrow::Borrow,
    fmt::{self, Formatter},
};

use super::ArgMatcher;

pub struct Eq<Expected>(Expected);

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

pub fn eq<Expected: fmt::Debug + PartialEq>(expected: Expected) -> Eq<Expected> {
    Eq(expected)
}

pub struct EqAgainst<Expected>(Expected);

pub fn eq_against<Expected>(expected: Expected) -> EqAgainst<Expected> {
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
