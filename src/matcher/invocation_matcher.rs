use core::fmt;
use paste::paste;

use super::ArgMatcher;

/// Matcher for invocation of a method.
///
/// Implementors provide an expectation for each method argument.
///
/// This trait is implemented for tuples of [`ArgMatcher`] of up to ten
/// elements.
pub trait InvocationMatcher<Args> {
    /// Returns `Ok(())` when all arguments match.
    ///
    /// Returns `Err(String)` if any argument fails to match. The
    /// error should detail which arguments failed and why.
    fn matches(&self, args: &Args) -> Result<(), String>;
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
    ($idx:tt, $($other:tt,)+) => (tuple! { $($other,)+ })
}

// implement InvocationMatcher for tuples for ArgMatchers
macro_rules! tuple {
    ($idx:tt,) => ();
    ($($idx:tt,)+) => (
        paste! {
            impl<$([<A $idx>]: fmt::Debug),+,$([<AM $idx>]: ArgMatcher<[<A $idx>]>),+> InvocationMatcher<($([<A $idx>],)+)> for ($([<AM $idx>],)+) {
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
        }
        peel! { $($idx,)+ }
    )
}

tuple! { 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, }
