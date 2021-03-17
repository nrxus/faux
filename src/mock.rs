use crate::matcher;
use std::fmt;

pub struct Mock<'a, I, O> {
    matcher: Box<dyn matcher::AllArgs<I> + Send>,
    stub: Stub<'a, I, O>,
}

pub enum Stub<'a, I, O> {
    Once(Box<dyn FnOnce(I) -> O + Send + 'a>),
    Many {
        stub: Box<dyn FnMut(I) -> O + Send + 'a>,
        times: MockTimes,
    },
}

pub struct SavedMock<'a> {
    transmuted_matcher: Box<dyn matcher::AllArgs<()> + Send>,
    stub: SavedStub<'a>,
}

pub enum SavedStub<'a> {
    Exhausted,
    Once {
        transmuted_stub: Box<dyn FnOnce(()) + Send + 'a>,
    },
    Many {
        transmuted_stub: Box<dyn FnMut(()) + Send + 'a>,
        times: MockTimes,
    },
}

#[derive(Debug)]
pub enum MockTimes {
    Always,
    Times(usize),
}

impl MockTimes {
    pub fn decrement(&mut self) {
        if let MockTimes::Times(times) = self {
            *times -= 1;
        }
    }
}

impl<'a, I, O> Mock<'a, I, O> {
    pub fn new<M: matcher::AllArgs<I> + Send + 'static>(stub: Stub<'a, I, O>, matcher: M) -> Self {
        Mock {
            matcher: Box::new(matcher),
            stub,
        }
    }

    pub unsafe fn unchecked(self) -> SavedMock<'a> {
        let matcher: Box<dyn matcher::AllArgs<()>> = std::mem::transmute(self.matcher);
        let stub = match self.stub {
            Stub::Once(mock) => SavedStub::Once {
                transmuted_stub: std::mem::transmute(mock),
            },
            Stub::Many { stub, times } => SavedStub::Many {
                times,
                transmuted_stub: std::mem::transmute(stub),
            },
        };
        SavedMock {
            transmuted_matcher: std::mem::transmute(matcher),
            stub,
        }
    }
}

impl<'a> SavedMock<'a> {
    /// # Safety
    ///
    /// Only call this method if you know for sure these are the right
    /// input and output from the non-transmuted stubs
    pub unsafe fn call<I, O>(&mut self, input: I) -> Result<O, String> {
        let matcher = &mut *(&mut self.transmuted_matcher as *mut Box<_>
            as *mut Box<dyn matcher::AllArgs<I>>);

        // TODO: should the error message be different if the stub is also exhausted?
        matcher.matches(&input)?;

        let just_exhausted = match &mut self.stub {
            SavedStub::Once { .. }
            | SavedStub::Many {
                times: MockTimes::Times(0),
                ..
            }
            | SavedStub::Many {
                times: MockTimes::Times(1),
                ..
            } => std::mem::replace(&mut self.stub, SavedStub::Exhausted),
            SavedStub::Many {
                times,
                transmuted_stub,
            } => {
                times.decrement();
                let stub = &mut *(transmuted_stub as *mut Box<dyn FnMut(()) + Send>
                    as *mut Box<dyn FnMut(I) -> O + Send>);
                return Ok(stub(input));
            }
            SavedStub::Exhausted => return Err("this mock has been exhausted".to_string()),
        };

        match just_exhausted {
            SavedStub::Once { transmuted_stub } => {
                let stub: Box<dyn FnOnce(I) -> O> = std::mem::transmute(transmuted_stub);
                Ok(stub(input))
            }
            SavedStub::Many {
                times: MockTimes::Times(0),
                ..
            } => Err("this mock has been exhausted".to_string()),
            SavedStub::Many {
                times: MockTimes::Times(1),
                transmuted_stub,
            } => {
                let mut stub: Box<dyn FnMut(I) -> O> = std::mem::transmute(transmuted_stub);
                Ok(stub(input))
            }
            _ => unreachable!(),
        }
    }
}

impl fmt::Debug for SavedMock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.stub {
            SavedStub::Exhausted => f.write_str("exhausted mock"),
            SavedStub::Once { .. } => f.write_str("once mock"),
            SavedStub::Many { times, .. } => write!(f, "mock {:?} times", times),
        }
    }
}
