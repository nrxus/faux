#![cfg_attr(doctest, feature(external_doc))]
#![allow(clippy::needless_doctest_main)]

//! # Faux
//!
//! A library to create [mocks] out of `struct`s.
//!
//! `faux` provides macros to help you create mocks out of your
//! structs without the need of generics/trait objects polluting your
//! function signatures.
//!
//! **`faux` makes liberal use of unsafe rust features, and it is only
//! recommended for use inside tests.**
//!
//! [mocks]: https://martinfowler.com/articles/mocksArentStubs.html
//!
//! ## Usage:
//! ```
//! // creates the mockable struct
//! #[faux::create]
//! pub struct Foo {
//!     a: u32,
//! }
//!
//! // mocks the methods
//! #[faux::methods]
//! impl Foo {
//!     pub fn new(a: u32) -> Self {
//!         Foo { a }
//!     }
//!
//!     pub fn add_stuff(&self, input: u32) -> u32 {
//!         self.a + input
//!     }
//!
//!     pub fn add_ref(&self, input: &u32) -> u32 {
//!         self.a + *input
//!     }
//! }
//!
//! fn main() {
//!   // you can create the original object
//!   let real = Foo::new(3);
//!   assert_eq!(real.add_stuff(2), 5);
//!
//!   // can create a mock using the auto-generated `faux` method
//!   let mut mock = Foo::faux();
//!
//!   // if the inputs and output for a method are all static types
//!   // then it can be mocked safely
//!   faux::when!(mock.add_stuff).safe_then(|x| x);
//!   assert_eq!(mock.add_stuff(5), 5);
//!
//!   // other methods can be mocked using unsafe
//!   unsafe { faux::when!(mock.add_ref).then(|&x| x + 1) }
//!   assert_eq!(mock.add_ref(&3), 4);
//! }
//! ```

mod mock;
mod mock_store;
mod when;

/// Transforms the given struct into a mockable version of itself.
///
/// It creates an associated function for the tagged struct called
/// `faux` and masks the original definition of the struct by changing
/// its name.
///
/// Use [cargo-expand] to see what your struct expands to after the
/// macro.
///
/// # Requirements
///
/// This macro deliberately fails to compile if any of the struct's
/// fields are not private. Otherwise, a user of the struct could try
/// to access the field directly when it no longer exists in the
/// transformed version.
///
/// The transformed struct is useless unless its methods are also
/// mocked. See [#\[methods\]] for documentation on how to mock the
/// methods of the struct. If `#[methods]` is not used for an impl
/// block, methods inside the impl may not use any of its fields.
///
/// # Known Limitations
///
/// [#9]: Mocked structs cannot have generic parameters
///
/// [#\[methods\]]: attr.methods.html
/// [cargo-expand]: https://github.com/dtolnay/cargo-expand
/// [#9]: https://github.com/nrxus/faux/issues/9
///
/// # Usage
/// ```
/// #[faux::create]
/// pub struct MyStruct {
///     a: i32,
///     b: Vec<u32>,
/// }
///
/// #[faux::methods]
/// impl MyStruct {
///     /* methods go here */
/// }
///
/// # fn main() {
/// // creates a mock out of MyStruct
/// let my_mock = MyStruct::faux();
/// # }
/// ```
pub use faux_macros::create;

/// Transforms the given methods into mockable versions of themselves
/// and provides a new method to mock them.
///
/// The generated methods look like
///
/// ```ignore
/// impl MyStruct {
///     /* other methods before */
///
///     // I is a tuple of all the non-receiver arguments of #{method_name}
///     // O is the output of #{method_name}
///     _when_#{method_name}(&mut self) -> When<I,O> {
///         /* auto generated code */
///     }
/// }
/// ```
///
/// These auto-generated methods can be called directly but a more
/// ergonomic way is by using [when!].
///
/// Associated functions and private methods are not mocked, and are
/// instead proxied to the real implementation.
///
/// # Requirements
///
/// [#\[create\]] must have been previously called for this struct.
///
/// # Known Limitations
/// [#10]: `impl SomeTrait for SomeStruct {}` is not supported.
///
/// [#11]: `impl path::to::SomeStruct {}` is not supported.
///
/// [#13]: Only a simple impl block may exist per module per type.
///
/// [#14]: Methods may not contain instances of the same struct as parameters.
///
/// [#\[create\]]: attr.create.html
/// [#10]: https://github.com/nrxus/faux/issues/10
/// [#11]: https://github.com/nrxus/faux/issues/11
/// [#13]: https://github.com/nrxus/faux/issues/13
/// [#14]: https://github.com/nrxus/faux/issues/14
/// [when!]: macro.when.html
///
/// # Usage
/// ```
/// #[faux::create]
/// pub struct MyStruct {
///     /* fields */
///     # data: Vec<u32>,
/// }
///
/// #[faux::methods]
/// impl MyStruct {
///     pub fn new(data: Vec<u32>) -> Self {
///         /* implementation code */
///         # MyStruct { data }
///     }
///
///     pub fn get(&self) -> usize {
///         20
///     }
/// }
///
/// # fn main() {
/// // #[methods]
/// let real = MyStruct::new(vec![5]);
/// assert_eq!(real.get(), 20);
///
/// // mock instances need to be mutable when mocking their methods
/// let mut fake = MyStruct::faux();
/// faux::when!(fake.get).safe_then(|_| 3);
/// assert_eq!(fake.get(), 3);
/// # }
/// ```
///
/// # Panics
///
/// ## Non-mocked methods
///
/// Faux will not try to return a default value on non-mocked methods
/// so it panics instead.
///
/// ```should_panic
/// #[faux::create]
/// pub struct MyStruct {}
///
/// #[faux::methods]
/// impl MyStruct {
///     pub fn get(&self) -> usize {
///         50
///     }
/// }
///
/// # fn main() {
/// let fake = MyStruct::faux();
/// // when!(fake.get).then_safe() was not invoked and thus the method was not mocked
/// fake.get(); // <~ panics with "'MyStruct::get' is not mocked"
/// # }
/// ```
///
/// ## Mocking real instances
///
/// Spies are not supported and thus mocking real instances panic.
///
/// ```should_panic
/// #[faux::create]
/// pub struct MyStruct {}
///
/// #[faux::methods]
/// impl MyStruct {
///     pub fn new() -> MyStruct {
///         MyStruct {}
///     }
///
///     pub fn get(&self) -> usize {
///         50
///     }
/// }
///
/// # fn main() {
/// let mut fake = MyStruct::new();
/// faux::when!(fake.get); // <~ panics with "not allowed to mock a real instance!"
/// # }
/// ```
///
/// # Caveats
/// ## Methods/functions that return the mocked struct
///
/// Special care is taken for methods and function that return an
/// instance of the mocked struct. Unfortunately only methods that
/// return `-> Self` or `-> #{SomeStruct}` are
/// handled.
///
/// Methods/functions that returns your type wrapped as a generic of
/// another type (e.g., `Result<Self, _>`) cannot be mocked.
///
/// ```compile_fail
/// #[faux::create]
/// pub struct MyStruct {}
///
/// #[faux::methods]
/// impl MyStruct {
///     pub fn try_to_new() -> Result<Self, String> {
///         Ok(MyStruct {})
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// A workaround is to place these functions outside the impl tagged
/// with `#[faux::method]` and have it redirect to the method inside the
/// tagged impl
///
/// ```
/// #[faux::create]
/// pub struct MyStruct {}
///
/// #[faux::methods]
/// impl MyStruct {
///     fn new() -> Self {
///         MyStruct {}
///     }
/// }
///
/// // do not tag this one
/// impl MyStruct {
///     pub fn try_to_new() -> Result<Self, String> {
///         Ok(MyStruct::new())
///     }
/// }
///
/// # fn main() {
/// let x = MyStruct::try_to_new();
/// assert!(x.is_ok());
/// # }
/// ```
///
/// ## Mocking struct defined elsewhere in the crate
///
/// Faux supports mocking structs from a different module as long as
/// we tell `#[methods]` where we are importing the struct from using
/// the `#[methods(path::to::module)]`
///
/// ```
/// mod foo {
///     #[faux::create]
///     pub struct MyStruct {}
///
///     // no need to tell #[faux::methods] where to find `MyStruct`
///     // if defined in the same module
///     #[faux::methods]
///     impl MyStruct {
///         pub fn three(&self) -> i32 {
///             3
///         }
///     }
///
///     mod foo_inner {
///         use super::MyStruct;
///
///         // we have to tell #[faux::methods] where we imported MyStruct from
///         #[faux::methods(super)]
///         impl MyStruct {
///             pub fn four(&self) -> i32 {
///                 self.three() + 1
///             }
///         }
///     }
/// }
///
/// mod bar {
///     use crate::foo::MyStruct;
///
///     // we have to tell #[faux::methods] where we imported MyStruct from
///     #[faux::methods(crate::foo)]
///     impl MyStruct {
///         pub fn five(&self) -> i32 {
///             self.three() + 2
///         }
///     }
/// }
///
/// # fn main() {
/// let mut x = foo::MyStruct::faux();
/// faux::when!(x.three).safe_then(|_| 30);
/// faux::when!(x.four).safe_then(|_| 40);
/// faux::when!(x.five).safe_then(|_| 50);
///
/// assert_eq!(x.three(), 30);
/// assert_eq!(x.four(), 40);
/// assert_eq!(x.five(), 50);
/// # }
/// ```
pub use faux_macros::methods;

/// Creates a [When] for a specific instance/method pair
///
/// This macro is a wrapper around calling the `_when_{method_name}()`
/// method that is auto-generated by [#\[methods\]].
///
/// [#\[methods\]]: attr.methods.html
/// [When]: When
///
/// ```
/// #[faux::create]
/// pub struct Foo {}
///
/// #[faux::methods]
/// impl Foo {
///     pub fn some_method(&self, a: u32, b: i8) -> i32 {
///         /* implementation code */
///         # panic!()
///     }
/// }
///
/// fn main() {
///     let mut mock = Foo::faux();
///     // input and output types are stored in the type signature of `When`
///     // calling `when!` or the auto-generated method creates the same `When`
///     let a: faux::When<(u32, i8), i32> = faux::when!(mock.some_method);
///     let b: faux::When<(u32, i8), i32> = mock._when_some_method();
/// }
/// ```
#[proc_macro_hack::proc_macro_hack]
pub use faux_macros::when;

pub use mock::Mock;
pub use mock_store::{MaybeFaux, MockStore};
pub use when::When;

#[doc(include = "../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
