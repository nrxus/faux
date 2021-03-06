mod any;
mod eq;

pub use any::any;
pub use eq::eq;

use std::fmt::{self, Formatter};

pub trait ArgMatcher<Arg> {
    type Message: fmt::Display;

    fn matches(&self, arg: &Arg) -> bool;
    fn message(&self) -> Self::Message;
}

impl<T: fmt::Debug> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "&{:?}", self.0)
    }
}

pub struct Ref<T>(pub T);

impl<T, A> PartialEq<A> for Ref<T>
where
    for<'a> A: PartialEq<&'a T>,
{
    fn eq(&self, rhs: &A) -> bool {
        *rhs == &self.0
    }
}

pub trait AllMatcher<Args> {
    fn matches(&self, args: &Args) -> Result<(), String>;
}

// needed to get around specialization
impl AllMatcher<()> for () {
    fn matches(&self, _: &()) -> Result<(), String> {
        Ok(())
    }
}

// needed to get around specialization
pub struct SingleMatcher<AM>(pub AM);
impl<Arg: fmt::Debug, AM: ArgMatcher<Arg>> AllMatcher<Arg> for SingleMatcher<AM> {
    fn matches(&self, arg: &Arg) -> Result<(), String> {
        if self.0.matches(arg) {
            Ok(())
        } else {
            Err(format!(
                "Argument did not match.\nExpected: {}\nActual: {:?}",
                self.0.message(),
                arg
            ))
        }
    }
}

impl<A: fmt::Debug, B: fmt::Debug, AM: ArgMatcher<A>, BM: ArgMatcher<B>> AllMatcher<(A, B)>
    for (AM, BM)
{
    fn matches(&self, (a, b): &(A, B)) -> Result<(), String> {
        let (am, bm) = &self;
        let matches = match (am.matches(a), bm.matches(b)) {
            (true, true) => return Ok(()),
            (a, b) => [a, b],
        };

        let expected = [am.message().to_string(), bm.message().to_string()];
        let actual = [format!("{:?}", a), format!("{:?}", b)];

        let argument_errors: Vec<_> = matches
            .iter()
            .enumerate()
            .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
            .map(|pos| {
                format!(
                    "Mismatched argument at position: {}.\nExpected: {}\nActual: {}",
                    pos, expected[pos], actual[pos]
                )
            })
            .collect();

        let argument_errors = argument_errors.join("\n\n");
        let expected = expected.join(", ");
        let actual = actual.join(", ");

        Err(format!(
            "Arguments did not match\n\nExpected: {}\nActual: {}\n\n{}",
            expected, actual, argument_errors
        ))
    }
}

impl<
        A: fmt::Debug,
        B: fmt::Debug,
        C: fmt::Debug,
        AM: ArgMatcher<A>,
        BM: ArgMatcher<B>,
        CM: ArgMatcher<C>,
    > AllMatcher<(A, B, C)> for (AM, BM, CM)
{
    fn matches(&self, (a, b, c): &(A, B, C)) -> Result<(), String> {
        let (am, bm, cm) = &self;
        let matches = match (am.matches(a), bm.matches(b), cm.matches(c)) {
            (true, true, true) => return Ok(()),
            (a, b, c) => [a, b, c],
        };

        let expected = [
            am.message().to_string(),
            bm.message().to_string(),
            cm.message().to_string(),
        ];
        let actual = [format!("{:?}", a), format!("{:?}", b), format!("{:?}", c)];

        let argument_errors: Vec<_> = matches
            .iter()
            .enumerate()
            .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
            .map(|pos| {
                format!(
                    "Mismatched argument at position: {}.\nExpected: {}\nActual: {}",
                    pos, expected[pos], actual[pos]
                )
            })
            .collect();

        let argument_errors = argument_errors.join("\n\n");
        let expected = expected.join(", ");
        let actual = actual.join(", ");

        Err(format!(
            "Arguments did not match\n\nExpected: {}\nActual: {}\n\n{}",
            expected, actual, argument_errors
        ))
    }
}
