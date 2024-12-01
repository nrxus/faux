use std::{
    fmt::{self, Formatter},
    num::NonZeroUsize,
};

use crate::matcher::InvocationMatcher;

pub struct Stub<'a, I, O> {
    matcher: Box<dyn InvocationMatcher<I> + Send>,
    answer: Answer<'a, I, O>,
}

pub enum Answer<'a, I, O> {
    Exhausted,
    Once(Box<dyn FnOnce(I) -> O + Send + 'a>),
    Many {
        stub: Box<dyn FnMut(I) -> O + Send + 'a>,
        times: Times,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Times {
    Always,
    Times(NonZeroUsize),
}

#[derive(Debug)]
pub enum Error {
    Exhausted,
    NotMatched(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Exhausted => f.write_str("stub was exhausted"),
            Error::NotMatched(error) => f.write_str(error),
        }
    }
}

impl Times {
    pub fn decrement(self) -> Option<Self> {
        match self {
            Times::Always => Some(self),
            Times::Times(n) => NonZeroUsize::new(n.get() - 1).map(Times::Times),
        }
    }
}

impl<'a, I, O> Stub<'a, I, O> {
    pub fn new(
        stub: Answer<'a, I, O>,
        matcher: impl InvocationMatcher<I> + Send + 'static,
    ) -> Self {
        Stub {
            matcher: Box::new(matcher),
            answer: stub,
        }
    }

    pub fn call(&mut self, input: I) -> Result<O, (I, Error)> {
        // TODO: should the error message be different if the stub is also exhausted?
        if let Err(e) = self.matcher.matches(&input) {
            return Err((input, Error::NotMatched(e)));
        }

        self.answer.call(input)
    }
}

impl<I, O> Answer<'_, I, O> {
    fn call(&mut self, input: I) -> Result<O, (I, Error)> {
        // no need to replace if we can keep decrementing
        if let Answer::Many { stub, times } = self {
            if let Some(decremented) = times.decrement() {
                *times = decremented;
                return Ok(stub(input));
            }
        }

        // otherwise replace it with an exhaust
        match std::mem::replace(self, Answer::Exhausted) {
            Answer::Exhausted => Err((input, Error::Exhausted)),
            Answer::Once(stub) => Ok(stub(input)),
            Answer::Many { mut stub, .. } => Ok(stub(input)),
        }
    }
}

impl<I, O> fmt::Debug for Stub<'_, I, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stub")
            // TODO: Add debug information for InvocationMatcher
            // .field("matcher", &self.matcher)
            .field(
                "answer",
                match &self.answer {
                    Answer::Exhausted => &"Exhausted",
                    Answer::Once(_) => &"Once",
                    Answer::Many { .. } => &"Many",
                },
            )
            .finish()
    }
}
