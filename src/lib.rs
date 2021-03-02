#![cfg_attr(doctest, feature(external_doc))]
#![allow(clippy::needless_doctest_main)]

//! # Faux
//!
//! A library to create [mocks] out of `struct`s.
//!
//! `faux` provides macros to help you create mocks out of your
//! structs without the need of generics nor trait objects polluting
//! your function signatures.
//!
//! **`faux` makes liberal use of unsafe rust features, and it is only
//! recommended for use inside tests.**
//!
//! [mocks]: https://martinfowler.com/articles/mocksArentStubs.html
//!
//! ## Usage
//!
//! ```
//! // creates the mockable struct
//! #[cfg_attr(test, faux::create)]
//! # #[faux::create]
//! pub struct Foo {
//!     a: u32,
//! }
//!
//! // mocks the methods
//! #[cfg_attr(test, faux::methods)]
//! # #[faux::methods]
//! impl Foo {
//!     pub fn new(a: u32) -> Self {
//!         Foo { a }
//!     }
//!
//!     pub fn add_stuff(&self, input: &u32) -> u32 {
//!         self.a + *input
//!     }
//!
//!     pub fn get_ref(&self) -> &u32 {
//!         &self.a
//!     }
//! }
//!
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   // you can create the original object
//!   let real = Foo::new(3);
//!   assert_eq!(real.add_stuff(&2), 5);
//!
//!   // can create a mock using the auto-generated `faux` method
//!   let mut mock = Foo::faux();
//!
//!   // safely mock return values of owned data
//!   faux::when!(mock.add_stuff).then_return(3);
//!   assert_eq!(mock.add_stuff(&20), 3);
//!
//!   // safely mock implementation of methods that return owned data
//!   faux::when!(mock.add_stuff).then(|&x| x);
//!   assert_eq!(mock.add_stuff(&5), 5);
//!
//!   // other methods can be mocked using unsafe
//!   let x = 5;
//!   unsafe { faux::when!(mock.get_ref).then_unchecked_return(&x) }
//!   assert_eq!(*mock.get_ref(), x);
//! }
//! #
//! # fn main() {
//! #  // you can create the original object
//! #  let real = Foo::new(3);
//! #  assert_eq!(real.add_stuff(&2), 5);
//! #
//! #   // can create a mock using the auto-generated `faux` method
//! #   let mut mock = Foo::faux();
//! #
//! #   // safely mock return values of owned data
//! #   faux::when!(mock.add_stuff).then_return(3);
//! #   assert_eq!(mock.add_stuff(&20), 3);
//! #
//! #   // saly mock implementation of methods that return owned data
//! #   faux::when!(mock.add_stuff).then(|&x| x);
//! #   assert_eq!(mock.add_stuff(&5), 5);
//! #
//! #   // other methods can be mocked using unsafe
//! #   let x = 5;
//! #   unsafe { faux::when!(mock.get_ref).then_unchecked_return(&x) }
//! #   assert_eq!(*mock.get_ref(), x);
//! #  }
//! ```
//!
//! ## Features:
//! * Mock async methods
//! * Mock trait implementations
//! * Mock generic structs
//! * Mock methods with arbitrary self types (e.g., `self: Rc<Self>`). **limited support**
//! * Mock methods from structs in a different module

mod mock;
mod mock_store;
mod when;

/// Transforms a struct into a mockable version of itself.
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
/// Only methods within `impl` blocks tagged by [#\[methods\]] may use
/// any of the struct fields.
///
/// # Usage
///
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
///
/// # Attribute arguments
///
/// ## self_type
///
/// Allowed values:
/// * `#[create(self_type = "Rc")]`
/// * `#[create(self_type = "Arc")]`
/// * `#[create(self_type = "Box")]`
/// * `#[create(self_type = "Owned")]`
///   * this is the default and not necessary
///
/// Indicates how to wrap the real value of the struct when not being
/// mocked, e.g., wrap an owned instance vs an `Rc<>`. `faux` will
/// guide to use this attribute when needed through either a compile
/// time error or a panic. Do not use unless `faux` asks you to.
///
/// Be default `faux` wraps owned instance of your struct (i.e.,
/// `MyStruct`). However, sometimes this is not ideal if the only
/// interactions for this struct are through a different self type
/// (e.g., `self: Rc<Mystruct>`). In this case, we can indicate `faux`
/// to hold a non-owned version (e.g., `Rc<MyStruct>`). This is
/// particularly useful if the only methods that return an instance of
/// the struct return a non-owned instance of it.
///
/// If this attribute is set, all of the `impl` blocks tagged with
/// [#\[methods\]] must specify the same `self_type`.
///
/// Although a `self_type` of `Pin` is not allowed, `faux` does allow
/// mocking of methods that take a `Pin<P>` as a receiver.
///
/// ### Usage
///
/// ```
/// use std::sync::Arc;
///
/// #[faux::create(self_type = "Arc")]
/// pub struct MyStruct {
///     /* private fields */
/// }
///
/// #[faux::methods(self_type = "Arc")]
/// impl MyStruct {
///     pub fn new() -> Arc<Self> {
///         /* implementation */
///         # Arc::new(MyStruct {})
///     }
///
///     /* more methods */
/// }
/// # fn main() {}
/// ```
///
/// [#\[methods\]]: attr.methods.html
/// [cargo-expand]: https://github.com/dtolnay/cargo-expand
///
pub use faux_macros::create;

/// Transforms methods in an `impl` block into mockable versions of
/// themselves.
///
/// The mockable methods can then be mocked using [when!].
///
/// Associated functions and private methods are not mocked, and are
/// instead proxied to the real implementation.
///
/// # Requirements
///
/// [#\[create\]] must have been previously called for this struct.
///
/// # Usage
///
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
/// use std::io::{self, Read};
///
/// #[faux::methods]
/// impl Read for MyStruct {
///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
///         /* potentially complicated implementation code */
///         # Ok(3)
///     }
/// }
///
/// # fn main() {
/// // #[methods] will not mock associated functions
/// // thus allowing you to still create real instances
/// let real = MyStruct::new(vec![5]);
/// assert_eq!(real.get(), 20);
///
/// // mock instances need to be mutable when mocking their methods
/// let mut fake = MyStruct::faux();
/// faux::when!(fake.get).then(|_| 3);
/// assert_eq!(fake.get(), 3);
/// // unsafe because a parameter is a reference.
/// // See When's documentation
/// unsafe { faux::when!(fake.read).then_unchecked(|a| Ok(a[0] as usize)) }
/// assert_eq!(fake.read(&mut vec![10]).unwrap(), 10);
/// # }
/// ```
/// # Attribute arguments
///
/// ## path
///
/// Indicates that the struct came from an imported module.
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
///         // the type is being imported from somewhere else
///         use super::MyStruct;
///
///         // so we have to tell faux where it came from
///         #[faux::methods(path = "super")]
///         impl MyStruct {
///             pub fn four(&self) -> i32 {
///                 self.three() + 1
///             }
///         }
///     }
/// }
///
/// mod bar {
///     // we are importing a module from somewhere else
///     use crate::foo;
///
///     // so we need to tell faux where that module came from
///     #[faux::methods(path = "crate")]
///     impl foo::MyStruct {
///         pub fn five(&self) -> i32 {
///             self.three() + 2
///         }
///     }
/// }
///
/// # fn main() {
/// let mut x = foo::MyStruct::faux();
/// faux::when!(x.three).then(|_| 30);
/// faux::when!(x.four).then(|_| 40);
/// faux::when!(x.five).then(|_| 50);
///
/// assert_eq!(x.three(), 30);
/// assert_eq!(x.four(), 40);
/// assert_eq!(x.five(), 50);
/// # }
/// ```
///
/// ## self_type
///
/// Allowed values:
/// * `#[methods(self_type = "Rc")]`
/// * `#[methods(self_type = "Arc")]`
/// * `#[methods(self_type = "Box")]`
/// * `#[methods(self_type = "Owned")]`
///   * this is the default and not necessary
///
/// Indicates how the real value of the struct is wrapped when not
/// being mocked, e.g., wrapped as an owned instance vs an
/// `Rc<>`. `faux` will guide to use this attribute when needed
/// through either a compile time error or a panic. Do not use unless
/// `faux` asks you to.
///
/// If this attribute is set, the [#\[create\]] attribute must specify
/// the same `self_type` in the struct.
///
/// By default `faux` assumes that it has access to an owned instance
/// of the struct. However, the [#\[create\]] macro may have a
/// `self_type` specified that wraps the instance differently. This is
/// useful when the method receivers are all the same non-owned
/// received (e.g., `self: Rc<Self>`).
///
/// The method receivers for all the methods in the impl block must be
/// convertable from the `self_type` specified. In particular, while a
/// `&self` can be obtained from an `Rc<Self>` or an `Arc<Self>`, a
/// `&mut self` cannot. This means that if you specify `self_type =
/// "Rc"`, then none of the methods being mocked may take a `&mut
/// self` as a receiver. If you believe that a certain combination of
/// specified `self_type` and method receiver is doable but not
/// allowed in `faux` please file an issue.
///
/// Another effect of specifying the `self_type` is gaining the
/// ability to include methods and associated functions that return
/// `Self` wrapped in that pointer type.
///
/// ```
/// use std::rc::Rc;
///
/// #[faux::create(self_type = "Rc")]
/// pub struct ByRc {}
///
/// #[faux::methods(self_type = "Rc")]
/// impl ByRc {
///     // you can return plain Self
///     pub fn new() -> Self {
///         ByRc {}
///     }
///
///     // or the Self wrapped in the self_type
///     pub fn new_rc() -> Rc<Self> {
///         Rc::new(ByRc {})
///     }
///
///     // can call methods with an Rc<Self> receiver type
///     pub fn by_rc(self: Rc<Self>) {}
///
///     // Rc<Self> derefs to &self so this is okay too
///     pub fn by_ref(&self) {}
/// }
/// # fn main() {}
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
/// // fake.get is not mocked
/// fake.get(); // <~ panics
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
/// faux::when!(fake.get); // <~ panics
/// # }
/// ```
///
/// ## Structs with `self: Rc<Self>` or `self: Arc<Self>` methods that have been cloned
///
/// While you do not need to specify `#[self_type = "Rc"]` or
/// `#[self_type = "Arc"]` even if you have `self: Rc<Self>` or `self:
/// Arc<Self>` receivers respectively, the real instances that are
/// created through this *must* be the only reference to the object
/// when calling these methods or else your test will fail.
///
/// ```rust should_panic
/// use std::rc::Rc;
///
/// #[faux::create]
/// pub struct Owned { /* fields */ }
///
/// #[faux::methods]
/// impl Owned {
///    pub fn new() -> Owned {
///        /* implementation */
///        # Owned {}
///    }
///
///    pub fn by_rc(self: Rc<Self>) {
///        /* implementation */
///    }
/// }
///
/// # pub fn main() {
/// // works if there is only a single reference
/// let rcd = Rc::new(Owned::new());
/// rcd.by_rc(); // this works
///
/// // panics if there are multiple references
/// let rcd = Rc::new(Owned::new());
/// let clone = rcd.clone();
/// rcd.by_rc(); // <~ panics: reference is not unique
/// # }
/// ```
///
/// In the case of a panic the panic message faux produces should
/// guide you towards using the `self_type` argument in the faux
/// attributes
///
/// # Known Limitations
///
/// [#13]: Only a single inherent impl block and a single trait
/// implementation per trait per type may exist.
///
/// [#14]: Methods may not contain instances of the same struct as
/// parameters.
///
/// [#18]: Generic methods and `impl` return types are not allowed
///
/// [#\[create\]]: attr.create.html
/// [#10]: https://github.com/nrxus/faux/issues/10
/// [#13]: https://github.com/nrxus/faux/issues/13
/// [#14]: https://github.com/nrxus/faux/issues/14
/// [#18]: https://github.com/nrxus/faux/issues/18
/// [when!]: macro.when.html
///
/// # Caveats
///
/// ## Methods/functions that return the mocked struct
///
/// Special care is taken for methods and function that return an
/// instance of the mocked struct. Unfortunately only methods that
/// return `-> Self` or `-> #{SomeStruct}` are
/// handled.
///
/// Methods/functions that returns your type wrapped as a generic of
/// another type (e.g., `Result<Self, _>`) cannot be wrapped in a faux
/// impl.  The exception to this is methods that receive an specified
/// [self_type](#self_type).
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
/// ## Paths in types
///
/// `faux` supports implementing types of the form `path::to::Type` as
/// long as the path does not contain any `super` or `crate`
/// keywords. To implement such types use the [`path`](#path)
/// argument.
pub use faux_macros::methods;

/// Creates a [When] instance to mock a specific method in a struct.
///
/// The method to mock must be be in an `impl` blocked tagged by
/// [#\[methods\]]
///
/// [#\[methods\]]: attr.methods.html
/// [When]: struct.When.html
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
///     // (u32, i8) is the input of the mocked method
///     // i32 is the output of the mocked method
///     let a: faux::When<(u32, i8), i32> = faux::when!(mock.some_method);
/// }
/// ```
pub use faux_macros::when;

pub use mock::ReturnedMock;
pub use mock_store::{MaybeFaux, MockStore};
pub use when::{When, WhenOnce};

#[doc(include = "../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
