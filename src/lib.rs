use std::{any::Any, collections::HashMap, cell::RefCell};

pub enum MaybeQuack<T> {
    Real(T),
    Quack(RefCell<Quack>),
}

impl<T> MaybeQuack<T> {
    pub fn quack() -> Self {
        MaybeQuack::Quack(RefCell::new(Quack::default()))
    }

    pub fn mock<I: 'static, O: 'static>(
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

#[derive(Default)]
pub struct Quack {
    mocks: HashMap<&'static str, Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>>,
}

impl Quack {
    pub fn mock<I: 'static, O: 'static>(
        &mut self,
        name: &'static str,
        mock: impl (FnOnce(I) -> O) + 'static,
    ) {
        let x: Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>> = Box::new(|input| {
            let input = *(input.downcast::<I>().unwrap());
            Box::new(mock(input)) as Box<dyn Any>
        });
        self.mocks.insert(name, x);
    }

    pub fn call_mock<I: 'static, O: 'static>(&mut self, name: &str, input: I) -> O {
        let mock = self
            .mocks
            .remove(name)
            .expect(&format!("no mock for method '{}'", name));
        *(mock(Box::new(input)).downcast::<O>().unwrap())
    }
}
