pub enum StoredMock<'a, I, O> {
    Once(Box<dyn FnOnce(I) -> O + Send + 'a>),
    Many(Box<dyn FnMut(I) -> O + Send + 'a>, MockTimes),
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
    pub fn once(mock: impl FnOnce(I) -> O + Send + 'a) -> Self {
        let mock = Box::new(mock);
        StoredMock::Once(mock)
    }

    pub fn many(mock: impl FnMut(I) -> O + Send + 'a, times: MockTimes) -> Self {
        let mock = Box::new(mock);
        StoredMock::Many(mock, times)
    }
}

pub enum UncheckedMock<'a> {
    Once(Box<dyn FnOnce(()) + Send + 'a>),
    Many(Box<dyn FnMut(()) + Send + 'a>, MockTimes),
}

impl<'a> UncheckedMock<'a> {
    pub unsafe fn new<I, O>(mock: StoredMock<'a, I, O>) -> Self {
        match mock {
            StoredMock::Once(mock) => UncheckedMock::Once(std::mem::transmute(mock)),
            StoredMock::Many(mock, times) => UncheckedMock::Many(std::mem::transmute(mock), times),
        }
    }

    pub unsafe fn transmute<I, O>(self) -> StoredMock<'a, I, O> {
        match self {
            UncheckedMock::Once(mock) => StoredMock::Once(std::mem::transmute(mock)),
            UncheckedMock::Many(mock, times) => StoredMock::Many(std::mem::transmute(mock), times),
        }
    }
}

use std::fmt;
impl fmt::Debug for UncheckedMock<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UncheckedMock::Once(_) => f.write_str("once mock"),
            UncheckedMock::Many(_, count) => write!(f, "mock {:?} times", count),
        }
    }
}
