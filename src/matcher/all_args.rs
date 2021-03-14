use core::fmt;
use paste::paste;

use super::ArgMatcher;

pub trait AllArgs<Args> {
    fn matches(&self, args: &Args) -> Result<(), String>;
}

impl AllArgs<()> for () {
    fn matches(&self, _: &()) -> Result<(), String> {
        Ok(())
    }
}

// needed to get around specialization
pub struct Single<AM>(pub AM);
impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> AllArgs<Arg> for Single<AM> {
    fn matches(&self, arg: &Arg) -> Result<(), String> {
        if self.0.matches(arg) {
            Ok(())
        } else {
            Err(format!(
                "Argument did not match.
Expected: {}
Actual:   {:?}",
                self.0.message(),
                arg
            ))
        }
    }
}

// macro for implementing n-ary tuple functions and operations
macro_rules! trues {
    ($($v:expr),*) => { ($(trues!(@true $v)),*) };
    (@true $v:expr) =>  { true };
}

macro_rules! peel {
    ($idx:tt, $($other:tt,)+) => (tuple! { $($other,)+ })
}

macro_rules! tuple {
    ($idx:tt,) => ();
    ($($idx:tt,)+) => (
        paste! {
            impl<$([<A $idx>]: fmt::Debug),+,$([<AM $idx>]: ArgMatcher<[<A $idx>]>),+> AllArgs<($([<A $idx>],)+)> for ($([<AM $idx>],)+) {
                fn matches(&self, ($([<a $idx>]),+): &($([<A $idx>],)+)) -> Result<(), String> {
                    let ($([<am $idx>]),+) = &self;
                    let matches = match ($([<am $idx>].matches([<a $idx>])),+) {
                        trues!($($idx),+) => return Ok(()),
                        ($([<a $idx>]),+) => [$([<a $idx>]),+],
                    };
                    let expected = [
                        $([<am $idx>].message().to_string()),+
                    ];
                    let actual = [
                        $(format!("{:?}", [<a $idx>])),+
                    ];
                    let widths = [
                        $(expected[$idx].len().max(actual[$idx].len())),+
                    ];
                    let expected = [
                        $(format!("{:>width$}", expected[$idx], width = widths[$idx])),+
                    ];
                    let actual = [
                        $(format!("{:>width$}", actual[$idx], width = widths[$idx])),+
                    ];

                    let argument_errors: Vec<_> = matches
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
                        .map(|pos| format!("Mismatched argument at position: {}
Expected: {}
Actual:   {}",
                            pos, expected[pos], actual[pos]
                        ))
                        .collect();

                    let argument_errors = argument_errors.join("\n\n");
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
