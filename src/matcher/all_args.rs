use core::fmt;

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

impl<A: fmt::Debug, B: fmt::Debug, AM: ArgMatcher<A>, BM: ArgMatcher<B>> AllArgs<(A, B)>
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
        let widths = [
            expected[0].len().max(actual[0].len()),
            expected[1].len().max(actual[1].len()),
        ];
        let expected = [
            format!("{:>width$}", expected[0], width = widths[0]),
            format!("{:>width$}", expected[1], width = widths[1]),
        ];
        let actual = [
            format!("{:>width$}", actual[0], width = widths[0]),
            format!("{:>width$}", actual[1], width = widths[1]),
        ];

        let argument_errors: Vec<_> = matches
            .iter()
            .enumerate()
            .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
            .map(|pos| {
                format!(
                    "Mismatched argument at position: {}
Expected: {}
Actual:   {}",
                    pos, expected[pos], actual[pos]
                )
            })
            .collect();

        let argument_errors = argument_errors.join("\n\n");
        let expected = expected.join(", ");
        let actual = actual.join(", ");

        Err(format!(
            "Arguments did not match
Expected: [{}]
Actual:   [{}]

{}",
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
    > AllArgs<(A, B, C)> for (AM, BM, CM)
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
        let widths = [
            expected[0].len().max(actual[0].len()),
            expected[1].len().max(actual[1].len()),
            expected[2].len().max(actual[2].len()),
        ];
        let expected = [
            format!("{:>width$}", expected[0], width = widths[0]),
            format!("{:>width$}", expected[1], width = widths[1]),
            format!("{:>width$}", expected[2], width = widths[2]),
        ];
        let actual = [
            format!("{:>width$}", actual[0], width = widths[0]),
            format!("{:>width$}", actual[1], width = widths[1]),
            format!("{:>width$}", actual[2], width = widths[2]),
        ];

        let argument_errors: Vec<_> = matches
            .iter()
            .enumerate()
            .filter_map(|(i, &passed)| if passed { None } else { Some(i) })
            .map(|pos| {
                format!(
                    "Mismatched argument at position: {}
Expected: {}
Actual:   {}",
                    pos, expected[pos], actual[pos]
                )
            })
            .collect();

        let argument_errors = argument_errors.join("\n\n");
        let expected = expected.join(", ");
        let actual = actual.join(", ");

        Err(format!(
            "Arguments did not match
Expected: [{}]
Actual:   [{}]

{}",
            expected, actual, argument_errors
        ))
    }
}
