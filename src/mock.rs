#[doc(hidden)]
pub enum StoredMock {
    Once(Box<dyn FnOnce(()) -> () + Send>),
    Many(Box<dyn FnMut(()) -> () + Send>, MockTimes),
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

#[doc(hidden)]
pub enum ReturnedMock<'a> {
    Once(Box<dyn FnOnce(()) -> () + Send>),
    Many(&'a mut (dyn FnMut(()) -> () + Send)),
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

impl ReturnedMock<'_> {
    /// # Safety
    ///
    /// [#[methods]] makes sure this function is called correctly when
    /// the method is invoked.  Do not use this function directly.
    ///
    /// [#\[methods\]]: methods
    pub unsafe fn call<I, O>(self, input: I) -> O {
        match self {
            ReturnedMock::Once(mock) => {
                let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(mock);
                mock(input)
            }
            ReturnedMock::Many(mock) => {
                let mock = &mut *(mock as *mut (dyn std::ops::FnMut(()) + std::marker::Send)
                    as *mut dyn std::ops::FnMut(I) -> O);
                mock(input)
            }
        }
    }
}
