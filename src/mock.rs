use crate::matcher;
use std::fmt;

pub enum StoredMock<'a, I, O> {
    Once(
        Box<dyn FnOnce(I) -> O + Send + 'a>,
        Box<dyn matcher::AllArgs<I>>,
    ),
    Many(
        Box<dyn FnMut(I) -> O + Send + 'a>,
        Box<dyn matcher::AllArgs<I>>,
        MockTimes,
    ),
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

impl<'a, I, O> StoredMock<'a, I, O> {
    pub fn once<M: matcher::AllArgs<I> + 'static>(
        mock: impl FnOnce(I) -> O + Send + 'a,
        matcher: M,
    ) -> Self {
        let mock = Box::new(mock);
        let matcher = Box::new(matcher);
        StoredMock::Once(mock, matcher)
    }

    pub fn many<M: matcher::AllArgs<I> + 'static>(
        mock: impl FnMut(I) -> O + Send + 'a,
        times: MockTimes,
        matcher: M,
    ) -> Self {
        let mock = Box::new(mock);
        let matcher = Box::new(matcher);
        StoredMock::Many(mock, matcher, times)
    }
}

pub enum UncheckedMock<'a> {
    Once(
        Box<dyn FnOnce(()) + Send + 'a>,
        Box<dyn matcher::AllArgs<()>>,
    ),
    Many(
        Box<dyn FnMut(()) + Send + 'a>,
        Box<dyn matcher::AllArgs<()>>,
        MockTimes,
    ),
}

impl<'a> UncheckedMock<'a> {
    pub unsafe fn new<I, O>(mock: StoredMock<'a, I, O>) -> Self {
        match mock {
            StoredMock::Once(mock, matcher) => {
                UncheckedMock::Once(std::mem::transmute(mock), std::mem::transmute(matcher))
            }
            StoredMock::Many(mock, matcher, times) => UncheckedMock::Many(
                std::mem::transmute(mock),
                std::mem::transmute(matcher),
                times,
            ),
        }
    }

    pub unsafe fn transmute<I, O>(self) -> StoredMock<'a, I, O> {
        match self {
            UncheckedMock::Once(mock, matcher) => {
                StoredMock::Once(std::mem::transmute(mock), std::mem::transmute(matcher))
            }
            UncheckedMock::Many(mock, matcher, times) => StoredMock::Many(
                std::mem::transmute(mock),
                std::mem::transmute(matcher),
                times,
            ),
        }
    }
}

impl fmt::Debug for UncheckedMock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UncheckedMock::Once(_, _) => f.write_str("once mock"),
            UncheckedMock::Many(_, _, count) => write!(f, "mock {:?} times", count),
        }
    }
}
