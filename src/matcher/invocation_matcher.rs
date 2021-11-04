use core::fmt;
use paste::paste;

use super::ArgMatcher;

/// Matcher for the invocation of a method.
///
/// Implementors provide an expectation for each method argument.
///
/// This trait is implemented for tuples of [`ArgMatcher`] of up to ten
/// elements.
///
/// # Examples
///
/// ## Simple
///
/// ```
/// use faux::matcher::{self, InvocationMatcher};
///
/// let matcher = (matcher::eq(5), matcher::any());
/// assert!(matcher.matches(&(5, "hello")).is_ok());
/// assert!(matcher.matches(&(3, "hello")).is_err());
/// ```
///
/// ## Single argument
///
/// ```
/// use faux::matcher::{self, InvocationMatcher};
///
/// // note that single arg matchers are wrapped in a tuple
/// // don't forget the trailing comma to denote it is a tuple
/// let matcher = (matcher::eq(20),);
/// assert!(matcher.matches(&20).is_ok());
/// ```
pub trait InvocationMatcher<Args, const N: usize> {
    /// Returns `Ok(())` when all arguments match.
    ///
    /// Returns `Err(String)` if any argument fails to match. The
    /// error should detail which arguments failed and why.
    fn matches(&self, args: &Args) -> Result<(), String>;
}

#[doc(hidden)]
pub struct AnyInvocation;

impl<Arg, const N: usize> InvocationMatcher<Arg, N> for AnyInvocation {
    /// Always returns Ok(())
    fn matches(&self, _: &Arg) -> Result<(), String> {
        Ok(())
    }
}

impl InvocationMatcher<(), 0> for () {
    /// Always succeeds, as there are no arguments to match against.
    fn matches(&self, _: &()) -> Result<(), String> {
        Ok(())
    }
}

impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> InvocationMatcher<Arg, 1> for (AM,) {
    /// Succeeds if the argument matches the [`ArgMatcher`].
    fn matches(&self, arg: &Arg) -> Result<(), String> {
        if self.0.matches(arg) {
            Ok(())
        } else {
            Err(format!(
                "Argument did not match.
Expected: {}
Actual:   {:?}",
                self.0, arg
            ))
        }
    }
}

// (1,2,3,..) => (true, true, true,..)
macro_rules! trues {
    ($($v:expr),*) => { ($(trues!(@true $v)),*) };
    (@true $v:expr) =>  { true };
}

// (a,b,c) => tuple!(b,c)
macro_rules! peel {
    ($idx:literal, $($other:literal),+) => (tuple! { $($other),+ })
}

// (a,b,c,...) => a
macro_rules! pop_front {
    ($first:tt, $($other:tt),*) => { $first }
}

// implement InvocationMatcher for tuples for ArgMatchers
macro_rules! tuple {
    ($idx:tt) => ();
    ($($idx:tt),+) => (
        paste! {
            impl<$([<A $idx>]: fmt::Debug),+,$([<AM $idx>]: ArgMatcher<[<A $idx>]>),+> InvocationMatcher<($([<A $idx>]),+), { pop_front! { $($idx),+ } }> for ($([<AM $idx>],)+) {
                /// Succeeds if every argument matches its corresponding [`ArgMatcher`].
                fn matches(&self, ($([<a $idx>]),+): &($([<A $idx>],)+)) -> Result<(), String> {
                    let ($([<am $idx>]),+) = &self;

                    let matches = match ($([<am $idx>].matches([<a $idx>])),+) {
                        trues!($($idx),+) => return Ok(()),
                        ($([<a $idx>]),+) => [$([<a $idx>]),+],
                    };

                    let expected = [
                        $([<am $idx>].to_string()),+
                    ];

                    let actual = [
                        $(format!("{:?}", [<a $idx>])),+
                    ];

                    Err(match_error(matches, expected, actual))
                }
            }
        }
        peel! { $($idx),+ }
    )
}

tuple! { 10, 9, 8, 7, 6, 5, 4, 3, 2, 1 }

fn match_error<const N: usize>(
    matches: [bool; N],
    mut expected: [String; N],
    mut actual: [String; N],
) -> String {
    expected
        .iter_mut()
        .zip(actual.iter_mut())
        .for_each(|(expected, actual)| {
            let width = expected.len().max(actual.len());

            *expected = format!("{:>width$}", expected, width = width);
            *actual = format!("{:>width$}", actual, width = width);
        });

    let argument_errors: Vec<_> = matches
        .into_iter()
        .enumerate()
        .filter_map(|(i, passed)| if passed { None } else { Some(i) })
        .map(|pos| {
            format!(
                "  Argument {}:
    Expected: {}
    Actual:   {}",
                pos, expected[pos], actual[pos]
            )
        })
        .collect();

    let argument_errors = argument_errors.join("\n");
    let expected = expected.join(", ");
    let actual = actual.join(", ");

    format!(
        "Arguments did not match
  Expected: [{}]
  Actual:   [{}]

{}",
        expected, actual, argument_errors
    )
}
