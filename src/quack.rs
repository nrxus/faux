use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct Quack {
    one_time_mocks: HashMap<TypeId, Box<dyn FnOnce(()) -> ()>>,
}

impl Quack {
    pub fn mock_once<I, O: 'static>(&mut self, id: TypeId, mock: impl FnOnce(I) -> O + 'static) {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = unsafe { std::mem::transmute(mock) };
        self.one_time_mocks.insert(id, mock);
    }

    /// Possible options:
    /// `id` was not previously stored: returns `Option::None`
    /// `id` was stored with an impl FnOnce(I) -> O: returns `Option::Some`
    /// `id` was stored with a different argument/output pair: UB
    pub unsafe fn call_mock<I, O: 'static>(&mut self, id: &TypeId, input: I) -> Option<O> {
        let mock = self.one_time_mocks.remove(&id)?;
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(mock);
        Some(mock(input))
    }
}

#[cfg(test)]
mod tests {}
