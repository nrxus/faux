use std::fmt;

use super::ArgMatcher;

pub struct Eq<Captured: fmt::Debug>(Captured);

impl<Arg, Captured: PartialEq<Arg> + fmt::Debug> ArgMatcher<Arg> for Eq<Captured> {
    type Message = String;

    fn matches(&self, arg: &Arg) -> bool {
        self.0 == *arg
    }

    fn message(&self) -> Self::Message {
        format!("{:?}", self.0)
    }
}

pub fn eq<Captured: fmt::Debug>(arg: Captured) -> Eq<Captured> {
    Eq(arg)
}
