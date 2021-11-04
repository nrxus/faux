use super::ArgMatcher;
use std::fmt::{self, Formatter};

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

/// Returns an [`ArgMatcher`] that succeeds if the provided closure
/// returns `true`.
///
/// The returned `Argmatcher` implements [`fmt::Display`] using the
/// string representation of the closure, so it is only recommended
/// for use with simple closures. For complex argument matching,
/// implement your own [`ArgMatcher`] to make the expectation message
/// more specific and less verbose.
///
/// # Examples
///
/// ```
/// use faux::{from_fn, matcher::ArgMatcher};
///
/// let contains_hello = from_fn!(|message: &str| message.contains("hello"));
/// assert!(contains_hello.matches("hello world"));
/// assert!(!contains_hello.matches("bye world"));
/// println!("{}", contains_hello); // '|message: &str| message.contains("hello")'
/// ```
///
/// ## Usage within when!
///
/// [`faux::when!`](crate::when!) does not have a special syntax for
/// this matcher. See the [matcher
/// syntax](macro.when.html#argument-matchers) for more
/// information.
///
/// ```ignore
/// // we can call it within `when!`
/// faux::when!(my_struct.some_method(_ = faux::from_fn!(|_: &i32| true)))
///     .then_return(5);
///
/// // or call outside `when!`
/// faux::when!(my_struct.some_method)
///     .with_args((faux::from_fn!(|_: &i32| true),)).then_return(5);
/// ```
#[macro_export]
macro_rules! from_fn {
    ($matcher:expr) => {
        faux::matcher::from_fn($matcher, stringify!($matcher))
    };
}

/// Returns an [`ArgMatcher`] that succeeds if the provided pattern
/// matches.
///
/// The returned `Argmatcher` implements [`fmt::Display`] using the
/// string representation of the pattern.
///
/// This macro has two forms:
/// * `pattern!(pattern)`
/// * `pattern!(type => pattern)`
///
/// Use the latter if you need to be specific about the type being
/// matched against.
///
/// # Examples
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
///
/// ## Usage within when!
///
/// [`faux::when!`](crate::when!) does not have a special syntax for
/// this matcher. See the [matcher
/// syntax](macro.when.html#argument-matchers) for more
/// information.
///
/// ```ignore
/// // we can call it within `when!`
/// faux::when!(my_struct.some_method(_ = faux::pattern!(|_: &i32| true)))
///     .then_return(5);
///
/// // or call outside `when!`
/// faux::when!(my_struct.some_method)
///     .with_args((faux::pattern!(|_: &i32| true),)).then_return(5);
#[macro_export]
macro_rules! pattern {
    ($(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => (
        faux::matcher::from_fn(
            move |arg| matches!(arg, $($pattern)|+ $(if $guard)?),
            stringify!($($pattern)|+ $(if $guard)?),
        )
    );
    ($ty:ty => $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => (
        faux::matcher::from_fn(
            move |arg: &$ty| matches!(arg, $($pattern)|+ $(if $guard)?),
            stringify!($($pattern)|+ $(if $guard)?),
        )
    );
}
