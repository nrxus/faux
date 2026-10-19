use core::fmt;

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
pub trait InvocationMatcher<Args> {
    /// Returns `Ok(())` when all arguments match.
    ///
    /// Returns `Err(String)` if any argument fails to match. The
    /// error should detail which arguments failed and why.
    fn matches(&self, args: &Args) -> Result<(), String>;
}

#[doc(hidden)]
pub struct AnyInvocation;

impl<Arg> InvocationMatcher<Arg> for AnyInvocation {
    /// Always returns Ok(())
    fn matches(&self, _: &Arg) -> Result<(), String> {
        Ok(())
    }
}

impl InvocationMatcher<()> for () {
    /// Always succeeds, as there are no arguments to match against.
    fn matches(&self, _: &()) -> Result<(), String> {
        Ok(())
    }
}

impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> InvocationMatcher<Arg> for (AM,) {
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
    ($first:tt, $($other:tt,)+) => (tuple! { $($other,)+ })
}

// implement InvocationMatcher for tuples for ArgMatchers
macro_rules! tuple {
    ($first:tt,) => ();
    ($(($idx:tt, $A:ident, $AM:ident, $a:ident, $am:ident),)+) => (
        impl<$($A: fmt::Debug),+,$($AM: ArgMatcher<$A>),+> InvocationMatcher<($($A,)+)> for ($($AM,)+) {
            /// Succeeds if every argument matches its corresponding [`ArgMatcher`].
            fn matches(&self, ($($a),+): &($($A,)+)) -> Result<(), String> {
                let ($($am),+) = &self;
                let matches = match ($($am.matches($a)),+) {
                    trues!($($idx),+) => return Ok(()),
                    ($($a),+) => [$($a),+],
                };
                let expected = [
                    $($am.to_string()),+
                ];
                let actual = [
                    $(format!("{:?}", $a)),+
                ];

                let mut widths = [
                    $(expected[$idx].len().max(actual[$idx].len())),+
                ];

                widths.reverse();

                let mut expected = [
                    $(format!("{:>width$}", expected[$idx], width = widths[$idx])),+
                ];

                expected.reverse();

                let mut actual = [
                    $(format!("{:>width$}", actual[$idx], width = widths[$idx])),+
                ];

                actual.reverse();

                let argument_errors: Vec<_> = matches
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
                    .map(|pos| format!("  Argument {}:
    Expected: {}
    Actual:   {}",
                        pos, expected[pos], actual[pos]
                    ))
                    .collect();

                let argument_errors = argument_errors.join("\n");
                let expected = expected.join(", ");
                let actual = actual.join(", ");

                Err(format!("Arguments did not match
  Expected: [{}]
  Actual:   [{}]

{}",
                    expected, actual, argument_errors
                ))
            }
        }
        peel! { $(($idx, $A, $AM, $a, $am),)+ }
    )
}

tuple! {
    (9, A9, AM9, a9, am9),
    (8, A8, AM8, a8, am8),
    (7, A7, AM7, a7, am7),
    (6, A6, AM6, a6, am6),
    (5, A5, AM5, a5, am5),
    (4, A4, AM4, a4, am4),
    (3, A3, AM3, a3, am3),
    (2, A2, AM2, a2, am2),
    (1, A1, AM1, a1, am1),
    (0, A0, AM0, a0, am0),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher;

    #[test]
    fn tuple_mismatch_message() {
        let matcher = (matcher::eq(100), matcher::any(), matcher::eq("x"));
        let err = matcher.matches(&(3, 42, "long value")).unwrap_err();
        assert_eq!(
            err,
            r#"Arguments did not match
  Expected: [100,  _,          "x"]
  Actual:   [  3, 42, "long value"]

  Argument 0:
    Expected: 100
    Actual:     3
  Argument 2:
    Expected:          "x"
    Actual:   "long value""#
        );
    }
}
