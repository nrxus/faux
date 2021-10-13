use crate::matcher::InvocationMatcher;
use std::fmt::{self, Formatter};

pub struct Stub<'a, I, O> {
    matcher: Box<dyn InvocationMatcher<I> + Send>,
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
    transmuted_matcher: Box<dyn InvocationMatcher<()> + Send>,
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

#[derive(Debug)]
pub enum Times {
    Always,
    Times(usize),
}

impl Times {
    pub fn decrement(&mut self) {
        if let Times::Times(times) = self {
            *times -= 1;
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
            stub,
        }
    }

    pub unsafe fn unchecked(self) -> Saved<'a> {
        let transmuted_matcher: Box<dyn InvocationMatcher<()> + Send> =
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
    pub unsafe fn call<I, O>(&mut self, input: I) -> Result<O, (I, String)> {
        let matcher = &mut *(&mut self.transmuted_matcher as *mut Box<_>
            as *mut Box<dyn InvocationMatcher<I>>);

        // TODO: should the error message be different if the stub is also exhausted?
        if let Err(e) = matcher.matches(&input) {
            return Err((input, e.to_string()));
        }

        let just_exhausted = match &mut self.stub {
            SavedAnswer::Once { .. }
            | SavedAnswer::Many {
                times: Times::Times(0),
                ..
            }
            | SavedAnswer::Many {
                times: Times::Times(1),
                ..
            } => std::mem::replace(&mut self.stub, SavedAnswer::Exhausted),
            SavedAnswer::Many {
                times,
                transmuted_stub,
            } => {
                times.decrement();
                let stub = &mut *(transmuted_stub as *mut Box<dyn FnMut(()) + Send>
                    as *mut Box<dyn FnMut(I) -> O + Send>);
                return Ok(stub(input));
            }
            SavedAnswer::Exhausted => {
                return Err((input, "this stub has been exhausted".to_string()))
            }
        };

        match just_exhausted {
            SavedAnswer::Once { transmuted_stub } => {
                let stub: Box<dyn FnOnce(I) -> O> = std::mem::transmute(transmuted_stub);
                Ok(stub(input))
            }
            SavedAnswer::Many {
                times: Times::Times(0),
                ..
            } => Err((input, "this stub has been exhausted".to_string())),
            SavedAnswer::Many {
                times: Times::Times(1),
                transmuted_stub,
            } => {
                let mut stub: Box<dyn FnMut(I) -> O> = std::mem::transmute(transmuted_stub);
                Ok(stub(input))
            }
            _ => unreachable!(),
        }
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
