mod quack;

pub use quack::Quack;

use std::{any::TypeId, cell::RefCell};

pub enum MaybeQuack<T> {
    Real(T),
    Quack(RefCell<Quack>),
}

impl<T> MaybeQuack<T> {
    pub fn quack() -> Self {
        MaybeQuack::Quack(RefCell::new(Quack::default()))
    }

    pub fn mock_once<I, O: 'static>(&mut self, id: TypeId, mock: impl (FnOnce(I) -> O) + 'static) {
        match self {
            MaybeQuack::Quack(quack) => quack.get_mut().mock_once(id, mock),
            MaybeQuack::Real(_) => panic!("not allowed to mock a real instance!"),
        }
    }
}
