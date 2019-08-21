use std::{any::TypeId, collections::HashMap};

#[derive(Default)]
pub struct Quack {
    mocks: HashMap<TypeId, Box<dyn FnOnce(()) -> ()>>,
}

impl Quack {
    pub fn mock<I, O: 'static>(&mut self, id: TypeId, mock: impl FnOnce(I) -> O + 'static) {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = unsafe { std::mem::transmute(mock) };
        self.mocks.insert(id, mock);
    }

    ///
    pub unsafe fn call_mock<I, O: 'static>(&mut self, id: &TypeId, input: I) -> Option<O> {
        let mock = self.mocks.remove(&id)?;
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(mock);
        Some(mock(input))
    }
}
