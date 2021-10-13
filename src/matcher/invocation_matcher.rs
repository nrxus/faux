use paste::paste;
use std::fmt::{self, Formatter};

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
    fn matches(&self, args: &Args) -> Result<(), Error>;

    fn expectations(&self) -> Vec<String>;
}

#[derive(Debug)]
pub struct ArgumentResult {
    expected: String,
    actual: String,
    matched: bool,
}

#[derive(Debug)]
pub enum Error {
    SingleArgument { expected: String, actual: String },
    Multiple(Vec<ArgumentResult>),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::SingleArgument { expected, actual } => write!(
                f,
                "Argument did not match.
  Expected: {}
  Actual:   {:?}",
                expected, actual
            ),
            Error::Multiple(arguments) => {
                assert!(!arguments.is_empty());

                let formatted_arguments: Vec<_> = arguments
                    .iter()
                    .map(
                        |ArgumentResult {
                             expected,
                             actual,
                             matched,
                         }| {
                            let width = expected.len().max(actual.len());
                            ArgumentResult {
                                expected: format!("{:>width$}", expected, width = width),
                                actual: format!("{:>width$}", actual, width = width),
                                matched: *matched,
                            }
                        },
                    )
                    .collect();

                writeln!(f, "Arguments did not match")?;

                // expected
                write!(f, "  Expected: [{}", formatted_arguments[0].expected)?;
                formatted_arguments
                    .iter()
                    .skip(1)
                    .try_for_each(|a| write!(f, ", {}", a.expected))?;
                writeln!(f, "]")?;

                // actual
                write!(f, "  Actual:   [{}", formatted_arguments[0].actual)?;
                formatted_arguments
                    .iter()
                    .skip(1)
                    .try_for_each(|a| write!(f, ", {}", a.actual))?;
                writeln!(f, "]")?;

                formatted_arguments
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| !a.matched)
                    .try_for_each(|(i, a)| {
                        write!(
                            f,
                            "
  Argument {}:
    Expected: {}
    Actual:   {}",
                            i, a.expected, a.actual
                        )
                    })?;

                Ok(())
            }
        }
    }
}

#[doc(hidden)]
pub struct AnyInvocation;

impl<Arg> InvocationMatcher<Arg> for AnyInvocation {
    /// Always returns Ok(())
    fn matches(&self, _: &Arg) -> Result<(), Error> {
        Ok(())
    }

    fn expectations(&self) -> Vec<String> {
        vec!["<any>".to_string()]
    }
}

impl InvocationMatcher<()> for () {
    /// Always succeeds, as there are no arguments to match against.
    fn matches(&self, _: &()) -> Result<(), Error> {
        Ok(())
    }

    fn expectations(&self) -> Vec<String> {
        vec![]
    }
}

impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> InvocationMatcher<Arg> for (AM,) {
    /// Succeeds if the argument matches the [`ArgMatcher`].
    fn matches(&self, arg: &Arg) -> Result<(), Error> {
        if self.0.matches(arg) {
            Ok(())
        } else {
            let mut expectations = self.expectations();
            Err(Error::SingleArgument {
                expected: expectations.pop().unwrap(),
                actual: format!("{:?}", arg),
            })
        }
    }

    fn expectations(&self) -> Vec<String> {
        vec![self.0.to_string()]
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
                fn expectations(&self) -> Vec<String> {
                    let ($([<am $idx>]),+) = &self;

                    vec![
                        $([<am $idx>].to_string()),+
                    ]
                }

                /// Succeeds if every argument matches its corresponding [`ArgMatcher`].
                fn matches(&self, ($([<a $idx>]),+): &($([<A $idx>],)+)) -> Result<(), Error> {
                    let ($([<am $idx>]),+) = &self;

                    let matches = match ($([<am $idx>].matches([<a $idx>])),+) {
                        trues!($($idx),+) => return Ok(()),
                        ($([<a $idx>]),+) => [$([<a $idx>]),+],
                    };

                    let expected = self.expectations();

                    let actual = [
                        $(format!("{:?}", [<a $idx>])),+
                    ];

                    let arguments = IntoIterator::into_iter(actual)
                        .zip(expected.into_iter())
                        .zip(IntoIterator::into_iter(matches))
                        .map(|((actual,expected),matched)| ArgumentResult {
                            actual,
                            expected,
                            matched,
                        })
                        .collect();

                    Err(Error::Multiple(arguments))
                }
            }
        }
        peel! { $($idx,)+ }
    )
}

tuple! { 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, }
