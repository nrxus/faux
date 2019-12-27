//! FAUX
//!
//! A library to create mocks out of `struct`s without polluting your
//! code with traits that exist for test only.
//!
//! This library makes liberal use of unsafe Rust features, and it is
//! not recommended for use outside of tests.
//!
//! Basic Usage:
//! ```edition2018
//! // `duck` marks a struct as mockable
//! #[faux::duck]
//! pub struct Foo {
//!     a: u32,
//! }
//!
//! // `quack` marks the methods inside an impl as methods to mock
//! #[faux::quack]
//! impl Foo {
//!     pub fn new(a: u32) -> Self {
//!         Foo { a }
//!     }
//!
//!     pub fn get_stuff(&self) -> u32 {
//!         self.a
//!     }
//! }
//!
//! fn main() {
//!   // `faux` will not override making the real version of your struct
//!   let real = Foo::new(3);
//!   assert_eq!(real.get_stuff(), 3);
//!
//!   // while providing a method to create a mock
//!   let mut mock = Foo::quack();
//!   unsafe { faux::when!(mock.get_stuff).then(|_| 10) }
//!   assert_eq!(mock.get_stuff(), 10);
//! }
//! ```

mod quack;

pub use faux_macros::{duck, quack};
use proc_macro_hack::proc_macro_hack;
pub use quack::Quack;
use std::{any::TypeId, cell::RefCell};

#[proc_macro_hack]
pub use faux_macros::when;

pub struct WhenHolder<'q, I, O> {
    pub id: TypeId,
    pub quack: &'q mut Quack,
    pub _marker: std::marker::PhantomData<(I, O)>,
}

impl<'q, I, O> WhenHolder<'q, I, O> {
    pub unsafe fn then(self, mock: impl FnOnce(I) -> O) {
        self.quack.mock_once(self.id, mock);
    }
}

#[doc(hidden)]
pub enum MaybeQuack<T> {
    Real(T),
    Quack(RefCell<Quack>),
}

impl<T> MaybeQuack<T> {
    pub fn quack() -> Self {
        MaybeQuack::Quack(RefCell::new(Quack::default()))
    }
}
