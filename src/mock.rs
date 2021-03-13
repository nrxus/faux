#[doc(hidden)]
pub enum StoredMock {
    Once(Box<dyn FnOnce(()) + Send>),
    Many(Box<dyn FnMut(()) + Send>, MockTimes),
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

impl std::fmt::Debug for StoredMock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StoredMock::Once(_) => f.write_str("once mock"),
            StoredMock::Many(_, count) => write!(f, "mock {:?} times", count),
        }
    }
}

impl StoredMock {
    pub(crate) unsafe fn once<I, O>(mock: impl FnOnce(I) -> O + Send) -> Self {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = std::mem::transmute(mock);
        StoredMock::Once(mock)
    }

    pub(crate) unsafe fn many<I, O>(mock: impl FnMut(I) -> O + Send, times: MockTimes) -> Self {
        let mock = Box::new(mock) as Box<dyn FnMut(_) -> _>;
        let mock = std::mem::transmute(mock);
        StoredMock::Many(mock, times)
    }
}
