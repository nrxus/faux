use core::fmt::{self, Formatter};
use paste::paste;

use super::ArgMatcher;

/// Matcher for the invocation of a method.
///
/// Implementors provide an expectation for each method argument.
///
/// This trait is implemented for tuples of [`ArgMatcher`] of up to
/// ten elements. All the methods inside are purposefully hidden and
/// the trait is sealed so any changes to its methods won't create a
/// breaking change. If you have an use-case where implementing this
/// trait would prove beneficial please submit an issue so the
/// decision of sealing this trait can be re-evaluated.
///
/// Do *NOT* rely on the signature of `InvocationMatcher`. While
/// removing implementations of `InvocationMatcher` will be considered
/// a breaking change, changing the signature (i.e., generics) of the
/// trait will not.
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
pub trait InvocationMatcher<Args, const N: usize>: private::Sealed {
    /// Returns `Ok(())` when all arguments match.
    ///
    /// Returns `Err(Error<N>)` if any argument fails to match. The
    /// error details what the actual arguments were and whether they
    /// matched or not. At least one of these argument must have not
    /// matched.
    #[doc(hidden)]
    fn matches(&self, args: &Args) -> Result<(), Error<N>>;

    #[doc(hidden)]
    /// Returns an array of a formmattted expectation, one per
    /// argument.
    fn expectations(&self) -> [String; N];
}

mod private {
    pub trait Sealed {}
}

#[derive(Debug)]
#[doc(hidden)]
pub struct Error<const N: usize> {
    arguments: [ArgumentMatch; N],
}

#[derive(Debug)]
struct ArgumentMatch {
    did_match: bool,
    actual: String,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct FormattedError<const N: usize> {
    arguments: [FormattedArgumentMatch; N],
}

#[derive(Debug)]
struct FormattedArgumentMatch {
    did_match: bool,
    expected: String,
    actual: String,
}

impl<const N: usize> Error<N> {
    pub fn formatted(self, expected: [String; N]) -> FormattedError<N> {
        let mut arguments =
            self.arguments.map(
                |ArgumentMatch { did_match, actual }| FormattedArgumentMatch {
                    did_match,
                    actual,
                    expected: String::new(),
                },
            );

        arguments
            .iter_mut()
            .zip(expected.into_iter())
            .for_each(|(arg, expected)| {
                let width = expected.len().max(arg.actual.len());

                arg.expected = format!("{:<width$}", expected, width = width);
                arg.actual = format!("{:<width$}", arg.actual, width = width);
            });

        FormattedError { arguments }
    }
}

impl<const N: usize> fmt::Display for FormattedError<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.arguments.len() == 1 {
            let FormattedArgumentMatch {
                expected, actual, ..
            } = &self.arguments[0];

            return write!(
                f,
                "Argument did not match.
  Expected: {}
  Actual:   {}",
                expected, actual
            );
        }

        f.write_str("Arguments did not match\n")?;

        write!(f, "  Expected: [{}", self.arguments[0].expected)?;
        self.arguments[1..]
            .iter()
            .try_for_each(|arg| write!(f, ", {}", arg.expected))?;
        f.write_str("]\n")?;

        write!(f, "  Actual:   [{}", self.arguments[0].actual)?;
        self.arguments[1..]
            .iter()
            .try_for_each(|arg| write!(f, ", {}", arg.actual))?;
        f.write_str("]\n")?;

        let mut mismatches = self
            .arguments
            .iter()
            .enumerate()
            .filter(|(_, arg)| !arg.did_match);

        if let Some((i, arg)) = mismatches.next() {
            write!(
                f,
                "  Argument {}:
    Expected: {}
    Actual:   {}",
                i, arg.expected, arg.actual
            )?;
        }

        mismatches.try_for_each(|(i, arg)| {
            write!(
                f,
                "\n  Argument {}:
    Expected: {}
    Actual:   {}",
                i, arg.expected, arg.actual
            )
        })
    }
}

impl<const N: usize> std::error::Error for FormattedError<N> {}

#[doc(hidden)]
pub struct AnyInvocation;

impl private::Sealed for AnyInvocation {}

impl<Arg, const N: usize> InvocationMatcher<Arg, N> for AnyInvocation {
    /// Always returns Ok(())
    fn matches(&self, _: &Arg) -> Result<(), Error<N>> {
        Ok(())
    }

    fn expectations(&self) -> [String; N] {
        ["<any>"; N].map(|e| e.to_string())
    }
}

impl InvocationMatcher<(), 0> for () {
    /// Always succeeds, as there are no arguments to match against.
    fn matches(&self, _: &()) -> Result<(), Error<0>> {
        Ok(())
    }

    fn expectations(&self) -> [String; 0] {
        []
    }
}

impl private::Sealed for () {}

impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> InvocationMatcher<Arg, 1> for (AM,) {
    /// Succeeds if the argument matches the [`ArgMatcher`].
    fn matches(&self, arg: &Arg) -> Result<(), Error<1>> {
        if self.0.matches(arg) {
            Ok(())
        } else {
            Err(Error {
                arguments: [ArgumentMatch {
                    did_match: true,
                    actual: format!("{:?}", arg),
                }],
            })
        }
    }

    fn expectations(&self) -> [String; 1] {
        [format!("{}", self.0)]
    }
}

impl<AM> private::Sealed for (AM,) {}

// (1,2,3,..) => (true, true, true,..)
macro_rules! trues {
    ($($v:expr),*) => { ($(trues!(@true $v)),*) };
    (@true $v:expr) =>  { true };
}

// (a,b,c) => tuple!(b,c)
macro_rules! peel {
    ($idx:tt, $($other:tt),+) => (tuple! { $($other),+ })
}

// (a,b,c,...) => a
macro_rules! pop_front {
    ($first:tt, $($other:tt),*) => {
        $first
    };
}

// implement InvocationMatcher for tuples for ArgMatchers
macro_rules! tuple {
    ($idx:tt) => ();
    ($($idx:tt),+) => (
        paste! {
            impl<$([<A $idx>]: fmt::Debug),+,$([<AM $idx>]: ArgMatcher<[<A $idx>]>),+> InvocationMatcher<($([<A $idx>]),+), { pop_front! { $($idx),+ } }> for ($([<AM $idx>],)+) {
                /// Succeeds if every argument matches its corresponding [`ArgMatcher`].
                fn matches(&self, ($([<a $idx>]),+): &($([<A $idx>],)+)) -> Result<(), Error<{ pop_front! { $($idx),+ } }>> {
                    let ($([<am $idx>]),+) = &self;

                    let matches = match ($([<am $idx>].matches([<a $idx>])),+) {
                        trues!($($idx),+) => return Ok(()),
                        ($([<a $idx>]),+) => [$([<a $idx>]),+],
                    };

                    let actual = [
                        $(format!("{:?}", [<a $idx>])),+
                    ];

                    // TODO: use array.zip(...).map(...)
                    // https://github.com/rust-lang/rust/issues/80094
                    let mut arguments = matches.map(|did_match| ArgumentMatch {
                        did_match,
                        actual: String::new(),
                    });

                    arguments
                        .iter_mut()
                        .zip(actual.into_iter()).
                        for_each(|(arg, actual)| {
                            arg.actual = actual;
                        });

                    Err(Error { arguments })
                }

                fn expectations(&self) -> [String; pop_front! { $($idx),+ }] {
                    let ($([<am $idx>]),+) = &self;

                    [
                        $([<am $idx>].to_string()),+
                    ]
                }
            }

            impl<$([<AM $idx>]),+> private::Sealed for ($([<AM $idx>],)+) {}
        }
        peel! { $($idx),+ }
    )
}

tuple! { 10, 9, 8, 7, 6, 5, 4, 3, 2, 1 }
