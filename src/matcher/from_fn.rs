use super::ArgMatcher;
use std::fmt;

pub struct Custom<F> {
    message: String,
    matcher: F,
}

impl<Arg, F: Fn(&Arg) -> bool> ArgMatcher<Arg> for Custom<F> {
    fn matches(&self, argument: &Arg) -> bool {
        let matcher = &self.matcher;
        matcher(argument)
    }
}

impl<F> fmt::Display for Custom<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

pub fn from_fn<Arg>(
    matcher: impl Fn(&Arg) -> bool,
    message: impl fmt::Display,
) -> impl ArgMatcher<Arg> {
    Custom {
        matcher,
        message: message.to_string(),
    }
}

#[macro_export]
macro_rules! from_fn {
    ($matcher:expr) => {
        faux::matcher::from_fn($matcher, stringify!($matcher))
    };
}

#[macro_export]
macro_rules! pattern {
    ($ty:ty => $( $pattern:pat )|+ $( if $guard: expr )? $(,)?) => (
        faux::matcher::from_fn(
            move |arg: &$ty| matches!(arg, $($pattern)|+ $(if $guard)?),
            stringify!($($pattern)|+ $(if $guard)?),
        )
    );
}
