use std::{any::Any, cell::RefCell, collections::HashMap};

pub enum MaybeQuack<T> {
    Real(T),
    Quack(RefCell<Quack>),
}

impl<T> MaybeQuack<T> {
    pub fn quack() -> Self {
        MaybeQuack::Quack(RefCell::new(Quack {
            mocks: HashMap::new(),
        }))
    }

    pub fn mock<I, O: 'static>(
        &mut self,
        name: &'static str,
        mock: impl (FnOnce(I) -> O) + 'static,
    ) {
        match self {
            MaybeQuack::Quack(quack) => quack.get_mut().mock(name, mock),
            MaybeQuack::Real(_) => panic!("not allowed to mock a real instance!"),
        }
    }
}

pub struct Quack {
    mocks: HashMap<&'static str, Box<dyn FnOnce(*mut ()) -> Box<dyn Any>>>,
}

impl Quack {
    pub fn mock<I, O: 'static>(&mut self, name: &'static str, mock: impl FnOnce(I) -> O + 'static) {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = unsafe { std::mem::transmute(mock) };
        self.mocks.insert(name, mock);
    }

    pub unsafe fn call_mock<I, O: 'static>(&mut self, name: &str, input: I) -> O {
        let mock = self
            .mocks
            .remove(name)
            .expect(&format!("no mock for method '{}'", name));
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(mock);
        mock(input)
    }
}
