use crate::matcher::InvocationMatcher;
use std::{
    fmt::{self, Formatter},
    num::NonZeroUsize,
};

pub struct Stub<'a, I, O, const N: usize> {
    matcher: Box<dyn InvocationMatcher<I, N> + Send>,
    stub: Answer<'a, I, O>,
}

pub enum Answer<'a, I, O> {
    Once(Box<dyn FnOnce(I) -> O + Send + 'a>),
    Many {
        stub: Box<dyn FnMut(I) -> O + Send + 'a>,
        times: Times,
    },
}

pub struct Saved<'a> {
    transmuted_matcher: Box<dyn InvocationMatcher<(), 0> + Send>,
    stub: SavedAnswer<'a>,
}

pub enum SavedAnswer<'a> {
    Exhausted,
    Once {
        transmuted_stub: Box<dyn FnOnce(()) + Send + 'a>,
    },
    Many {
        transmuted_stub: Box<dyn FnMut(()) + Send + 'a>,
        times: Times,
    },
}

impl<'a> SavedAnswer<'a> {
    unsafe fn call<I, O>(&mut self, input: I) -> Result<O, (I, String)> {
        unsafe fn call_transmuted_fnmut<'a, I, O>(
            transmuted_stub: &mut Box<dyn FnMut(()) + Send + 'a>,
            input: I,
        ) -> O {
            let stub = &mut *(transmuted_stub as *mut Box<dyn FnMut(()) + Send>
                as *mut Box<dyn FnMut(I) -> O + Send>);
            stub(input)
        }

        // no need to replace if we can keep decrementing
        if let SavedAnswer::Many {
            transmuted_stub,
            times,
        } = self
        {
            if let Some(decremented) = times.decrement() {
                *times = decremented;
                return Ok(call_transmuted_fnmut(transmuted_stub, input));
            }
        }

        // otherwise replace it with an exhaust
        match std::mem::replace(self, SavedAnswer::Exhausted) {
            SavedAnswer::Exhausted => Err((input, "this stub has been exhausted".to_string())),
            SavedAnswer::Once { transmuted_stub } => {
                let stub: Box<dyn FnOnce(I) -> O> = std::mem::transmute(transmuted_stub);
                Ok(stub(input))
            }
            SavedAnswer::Many {
                mut transmuted_stub,
                ..
            } => Ok(call_transmuted_fnmut(&mut transmuted_stub, input)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Times {
    Always,
    Times(NonZeroUsize),
}

impl Times {
    pub fn decrement(self) -> Option<Self> {
        match self {
            Times::Always => Some(self),
            Times::Times(n) => NonZeroUsize::new(n.get() - 1).map(Times::Times),
        }
    }
}

impl<'a, I, O, const N: usize> Stub<'a, I, O, N> {
    pub fn new(
        stub: Answer<'a, I, O>,
        matcher: impl InvocationMatcher<I, N> + Send + 'static,
    ) -> Self {
        Stub {
            matcher: Box::new(matcher),
            stub,
        }
    }

    pub unsafe fn unchecked(self) -> Saved<'a> {
        let transmuted_matcher: Box<dyn InvocationMatcher<(), 0> + Send> =
            std::mem::transmute(self.matcher);
        let stub = match self.stub {
            Answer::Once(stub) => SavedAnswer::Once {
                transmuted_stub: std::mem::transmute(stub),
            },
            Answer::Many { stub, times } => SavedAnswer::Many {
                times,
                transmuted_stub: std::mem::transmute(stub),
            },
        };
        Saved {
            transmuted_matcher,
            stub,
        }
    }
}

impl<'a> Saved<'a> {
    /// # Safety
    ///
    /// Only call this method if you know for sure these are the right
    /// input and output from the non-transmuted stubs
    pub unsafe fn call<I, O, const N: usize>(&mut self, input: I) -> Result<O, (I, String)> {
        let matcher = &mut *(&mut self.transmuted_matcher as *mut Box<_>
            as *mut Box<dyn InvocationMatcher<I, N>>);

        // TODO: should the error message be different if the stub is also exhausted?
        if let Err(e) = matcher.matches(&input) {
            return Err((input, e.formatted(matcher.expectations()).to_string()));
        }

        self.stub.call(input)
    }
}

impl fmt::Debug for Saved<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.stub {
            SavedAnswer::Exhausted => f.write_str("exhausted stub"),
            SavedAnswer::Once { .. } => f.write_str("once stub"),
            SavedAnswer::Many { times, .. } => write!(f, "stub {:?} times", times),
        }
    }
}
