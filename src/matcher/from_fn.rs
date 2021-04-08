use super::ArgMatcher;
use std::fmt;

struct FromFn<F> {
    message: String,
    matcher: F,
}

impl<Arg, F> ArgMatcher<Arg> for FromFn<F>
where
    Arg: ?Sized,
    F: Fn(&Arg) -> bool,
{
    fn matches(&self, argument: &Arg) -> bool {
        let matcher = &self.matcher;
        matcher(argument)
    }
}

impl<F> fmt::Display for FromFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

#[doc(hidden)]
pub fn from_fn<Arg>(
    matcher: impl Fn(&Arg) -> bool,
    message: impl fmt::Display,
) -> impl ArgMatcher<Arg>
where
    Arg: ?Sized,
{
    FromFn {
        matcher,
        message: message.to_string(),
    }
}

/// Returns an [`ArgMatcher`] that succeeds based on the provided
/// closure.
///
/// The returned `Argmatcher` implements [`fmt::Display`] using the
/// string representation of the closure.
///
/// This is only meant to be used for simple closures. For complex
/// argument matching implement your own [`ArgMatcher`] to make the
/// expectation message more specific and less verbose.
///
/// ```
/// use faux::{from_fn, matcher::ArgMatcher};
///
/// let contains_hello = from_fn!(|message: &str| message.contains("hello"));
/// assert!(contains_hello.matches("hello world"));
/// assert!(!contains_hello.matches("bye world"));
/// println!("{}", contains_hello); // '|message: &str| message.contains("hello")'
/// ```
#[macro_export]
macro_rules! from_fn {
    ($matcher:expr) => {
        faux::matcher::from_fn($matcher, stringify!($matcher))
    };
}

/// Returns an [`ArgMatcher`] that succeeds if the pattern matches
///
/// The returned `Argmatcher` implements [`fmt::Display`] using the
/// string representation of pattern.
///
/// This macro has two forms:
/// * `pattern!(pattern)`
/// * `pattern!(type => pattern)`
///
/// Use the latter to be specific about the type being matched
/// against.
///
/// ```
/// use faux::{pattern, matcher::ArgMatcher};
///
/// // the type can be implicit
/// let is_alphabet = pattern!('A'..='Z' | 'a'..='z');
/// assert!(is_alphabet.matches(&'f'));
/// assert!(!is_alphabet.matches(&' '));
///
/// // or the type can be explicit
/// let exists_more_than_two = pattern!(Option<_> => Some(x) if *x > 2);
/// assert!(exists_more_than_two.matches(&Some(4)));
/// assert!(!exists_more_than_two.matches(&Some(1)));
///
/// println!("{}", exists_more_than_two); // 'Some(x) if *x > 2'
/// ```
#[macro_export]
macro_rules! pattern {
    ($( $pattern:pat )|+ $( if $guard: expr )? $(,)?) => (
        faux::matcher::from_fn(
            move |arg| matches!(arg, $($pattern)|+ $(if $guard)?),
            stringify!($($pattern)|+ $(if $guard)?),
        )
    );
    ($ty:ty => $( $pattern:pat )|+ $( if $guard: expr )? $(,)?) => (
        faux::matcher::from_fn(
            move |arg: &$ty| matches!(arg, $($pattern)|+ $(if $guard)?),
            stringify!($($pattern)|+ $(if $guard)?),
        )
    );
}
