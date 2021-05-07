#![cfg_attr(doctest, feature(external_doc))]
#![allow(clippy::needless_doctest_main)]

//! A library to create [mocks] out of structs.
//!
//! `faux` allows you to mock the methods of structs for testing
//! without complicating or polluting your code.
//!
//! Part of `faux`'s philosophy is that only visible behavior should
//! be mocked. In practice, this means `faux` only mocks public
//! methods. Fields are not mocked, as they are data, not
//! behavior. Private methods are not mocked, as they are invisible to
//! others.
//!
//! At a high level, `faux` is split into:
//!
//! * [`#[create]`](create): transforms a struct into a mockable equivalent
//! * [`#[methods]`](methods): transforms the methods in an `impl` block into
//! their mockable equivalents
//! * [`when!`]: initializes a method stub by returning a [`When`]. Passing optional argument matchers restricts which arguments will invoke the mock.
//! * [`When`]: lets you stub a mocked method's return value or implementation
//!
//! # Getting Started
//!
//! `faux` makes liberal use of unsafe Rust features, so it is only
//! recommended for use inside tests. To prevent `faux` from leaking
//! into your production code, set it as a `dev-dependency` in your
//! `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! faux = "^0.1"
//! ```
//!
//! # Examples
//!
//! ## Simple
//!
//! ```
//! // restrict faux to tests by using `#[cfg_attr(test, ...)]`
//! // faux::create makes a struct mockable and generates an
//! // associated `faux()` function
//! // e.g.: `HttpClient::faux()` will create a mock `HttpClient` instance
//! #[cfg_attr(test, faux::create)]
//! # #[faux::create]
//! pub struct HttpClient { /* */ }
//!
//! // this is just a bag of data with no behavior
//! // so we do not attach `#[faux::create]`
//! #[derive(PartialEq, Clone, Debug)]
//! pub struct Headers {
//!     pub authorization: String,
//! }
//!
//! // `faux::methods` makes every public method in the `impl` block mockable
//! #[cfg_attr(test, faux::methods)]
//! # #[faux::methods]
//! impl HttpClient {
//!     pub fn post(&self, path: &str, headers: &Headers) -> String {
//!         /* makes network calls that we'd rather not do in unit tests */
//!         # unreachable!()
//!     }
//! }
//!
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   // use the generated `faux()` function to create a mock instance
//!   let mut mock = HttpClient::faux();
//!
//!   let headers = Headers { authorization: "Bearer foobar".to_string() };
//!
//!   // use `faux::when!` to mock the behavior of your methods
//!   // you can specify arguments to match against when the mock is invoked
//!   faux::when!(
//!       // arguments are converted into argument matchers
//!       // the default argument matcher performs an equality check
//!       // use `_` to create a universal argument matcher
//!       // the argument matchers below specify to ignore the first argument
//!       // but that the second one must equal `headers`
//!       mock.post(_, headers.clone())
//!   )
//!   // mock the return value
//!   .then_return("{}".to_string());
//!
//!   assert_eq!(mock.post("any/path/does/not/mater", &headers), "{}");
//!   assert_eq!(mock.post("as/i/said/does/not/matter", &headers), "{}");
//!
//!   // if you want to mock all calls to a method, you can omit argument matchers
//!   faux::when!(mock.post).then_return("OK".to_string());
//!   let other_headers = Headers { authorization: "other-token".to_string() };
//!   assert_eq!(mock.post("other/path", &other_headers), "OK");
//! }
//! #
//! # fn main() {
//! #   let mut mock = HttpClient::faux();
//! #   let headers = Headers { authorization: "Bearer foobar".to_string() };
//! #
//! #   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//! #   assert_eq!(mock.post("any/path/does/not/mater", &headers), "{}");
//! #   assert_eq!(mock.post("as/i/said/does/not/matter", &headers), "{}");
//! #
//! #   faux::when!(mock.post).then_return("OK".to_string());
//! #   let other_headers = Headers { authorization: "other-token".to_string() };
//! #   assert_eq!(mock.post("other/path", &other_headers), "OK");
//! #  }
//! ```
//!
//! ## Mocking the same method multiple times
//!
//! A single method can be mocked multiple times. When doing so,
//! `faux` checks every mock for the method in a last-in-first-out
//! fashion until it finds a mock whose argument matchers match the
//! invocation arguments.
//!
//! ```
//! # #[faux::create]
//! # pub struct HttpClient { /* */ }
//! # #[derive(PartialEq, Clone, Debug)]
//! # pub struct Headers {
//! #     pub authorization: String,
//! # }
//! # #[faux::methods]
//! # impl HttpClient {
//! #     pub fn post(&self, path: &str, headers: &Headers) -> String {
//! #         unreachable!()
//! #     }
//! # }
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   let mut mock = HttpClient::faux();
//!   let headers = Headers { authorization: "Bearer foobar".to_string() };
//!   let other_headers = Headers { authorization: "other-token".to_string() };
//!
//!   // catch-all mock to return "OK"
//!   faux::when!(mock.post).then_return("OK".to_string());
//!   // mock for specific headers to return "{}"
//!   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//!
//!   assert_eq!(mock.post("some/path", &headers), "{}"); // matches specific mock
//!   assert_eq!(mock.post("some/path", &other_headers), "OK"); // matches catch-all mock
//! }
//! # fn main() {
//! #   let mut mock = HttpClient::faux();
//! #   let headers = Headers { authorization: "Bearer foobar".to_string() };
//! #   faux::when!(mock.post).then_return("OK".to_string());
//! #   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//! #
//! #   assert_eq!(mock.post("any/path/does/not/mater", &headers), "{}");
//! #   assert_eq!(mock.post("as/i/said/does/not/matter", &headers), "{}");
//! #   let other_headers = Headers { authorization: "other-token".to_string() };
//! #   assert_eq!(mock.post("other/path", &other_headers), "OK");
//! #  }
//! ```
//!
//! ## Mocking implementation
//!
//! `faux` supports stubbing of not just the return value but also the
//! implementation of a method. This is done using `then()`.
//!
//! ```
//! # #[faux::create]
//! # pub struct HttpClient { /* */ }
//! # #[derive(PartialEq, Clone, Debug)]
//! # pub struct Headers {
//! #     pub authorization: String,
//! # }
//! # #[cfg_attr(test, faux::methods)]
//! # #[faux::methods]
//! # impl HttpClient {
//! #     pub fn post(&self, path: &str, headers: &Headers) -> String {
//! #        unreachable!()
//! #     }
//! # }
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   let mut mock = HttpClient::faux();
//!   let headers = Headers { authorization: "Bearer foobar".to_string() };
//!
//!   faux::when!(mock.post).then(|(path, _)| path.to_string().to_uppercase());
//!   assert_eq!(mock.post("another/path", &headers), "ANOTHER/PATH");
//! }
//! #
//! # fn main() {
//! #   let mut mock = HttpClient::faux();
//! #   let headers = Headers { authorization: "Bearer foobar".to_string() };
//! #   faux::when!(mock.post).then(|(path, _)| path.to_string().to_uppercase());
//! #   assert_eq!(mock.post("another/path", &headers), "ANOTHER/PATH");
//! #  }
//! ```
//!
//! ## Mocking with non-static data
//!
//! Let's add a new method to our `HttpClient` that returns borrowed
//! data. This cannot be mocked using safe code, so `faux` provides
//! `.then_unchecked()` and `.then_unchecked_return()` to mock such
//! methods.
//!
//! ```
//! # #[faux::create]
//! # pub struct HttpClient { /* */ }
//! #[cfg_attr(test, faux::methods)]
//! # #[faux::methods]
//! impl HttpClient {
//!     pub fn host(&self) -> &str {
//!         /* returns a reference to some internal data */
//!         # unreachable!()
//!     }
//! }
//!
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   let mut mock = HttpClient::faux();
//!
//!   // `then_unchecked()` and `then_unchecked_return()` require unsafe
//!   // they allow mocking methods that return non-static values (e.g. references)
//!   // or to mock using non-static closures
//!   let ret = "some-value".to_string();
//!   unsafe { faux::when!(mock.host).then_unchecked_return(ret.as_str()) }
//!   assert_eq!(mock.host(), &ret);
//! }
//! #
//! # fn main() {
//! #   let mut mock = HttpClient::faux();
//! #   let ret = "some-value".to_string();
//! #   unsafe { faux::when!(mock.host).then_unchecked_return(ret.as_str()) }
//! #   assert_eq!(mock.host(), &ret);
//! #  }
//! ```
//!
//! # Features
//!
//! `faux` lets you mock the return value or implementation of:
//!
//! * Async methods
//! * Trait methods
//! * Generic struct methods
//! * Methods with pointer self types (e.g., `self: Rc<Self>`)
//! * Methods in external modules
//!
//! `faux` also provides easy-to-use argument matchers.
//!
//! [mocks]: https://martinfowler.com/articles/mocksArentStubs.html

mod mock;
mod mock_store;

pub mod matcher;
pub mod when;

/// Transforms a struct into a mockable version of itself.
///
/// An associated function called `faux` is created for the tagged
/// struct, masking the original definition of the struct by changing
/// its name.
///
/// Use [`cargo-expand`] to see the changes to your struct after macro
/// expansion.
///
/// # Requirements
///
/// This macro deliberately fails to compile if any of the struct's
/// fields are not private. Otherwise, a user of the struct could try
/// to access the field directly when it no longer exists in the
/// transformed version.
///
/// Only methods within `impl` blocks tagged by
/// [`#[methods]`](methods) may use the struct's fields.
///
/// # Examples
///
/// ```
/// #[cfg_attr(test, faux::create)]
/// # #[faux::create]
/// pub struct MyStruct {
///     a: i32,
///     b: Vec<u32>,
/// }
///
/// #[cfg_attr(test, faux::methods)]
/// # #[faux::methods]
/// impl MyStruct {
///     /* methods go here */
/// }
///
/// # fn main() {
/// // creates a mock instance of MyStruct
/// let my_mock = MyStruct::faux();
/// # }
/// ```
///
/// # Attribute arguments
///
/// ## self_type
///
/// Customizes storage of real instances of the mockable struct.
///
/// If `self_type` is set, it must be set to the same value in all
/// [`#[methods]`](methods) for this struct.
///
/// ### Explanation
///
/// By default, `#[faux::create]` transform a struct from:
///
/// ```rust
/// #[faux::create]
/// struct MyStruct { /* fields */ }
/// ```
///
/// into something like:
///
/// ```
/// struct MyStruct(MaybeFaux);
///
/// enum MaybeFaux {
///   // when a mock is created we use this variant
///   Mock(/* snip */),
///   // when a real instance is created we use this variant
///   // stores an owned instance of the struct
///   // not a smart pointer to the struct
///   Real(OriginalMyStruct),
/// }
///
/// // the definition of the original struct
/// struct OriginalMyStruct { /* fields */ }
/// ```
///
/// This works well when constructors of `MyStruct` returns an owned
/// instance. There are some cases, however, where the constructor
/// returns a smart pointer (i.e., `Rc<Self>`). To support such cases,
/// use this attribute argument to customize how `faux` will wrap the
/// real instance of your struct.
///
/// ### Examples
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
/// ### Allowed values:
/// * `#[create(self_type = "Owned")]` (default)
/// * `#[create(self_type = "Rc")]`
/// * `#[create(self_type = "Arc")]`
/// * `#[create(self_type = "Box")]`
///
/// [`cargo-expand`]: https://github.com/dtolnay/cargo-expand
///
pub use faux_macros::create;

/// Transforms methods in an `impl` block into mockable versions of
/// themselves.
///
/// Mockable methods can be mocked using [`when!`].
///
/// Associated functions and private methods cannot be mocked. Calls
/// to them are proxied to the real implementation.
///
/// # Requirements
///
/// The struct definition must have been tagged with
/// [`#[create]`](create).
///
/// # Examples
///
/// ```
/// // #[faux::create] is a pre-req of #[faux::methods]
/// #[cfg_attr(test, faux::create)]
/// # #[faux::create]
/// pub struct MyStruct {
///     /* fields */
///     # data: Vec<u32>,
/// }
///
/// // make every public method mockable
/// #[cfg_attr(test, faux::methods)]
/// # #[faux::methods]
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
/// // make every method mockable
/// #[cfg_attr(test, faux::methods)]
/// # #[faux::methods]
/// impl Read for MyStruct {
///     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
///         /* implementation code */
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
/// // mock instances need to be mutable
/// let mut fake = MyStruct::faux();
/// faux::when!(fake.get).then_return(3);
/// assert_eq!(fake.get(), 3);
///
/// faux::when!(fake.read).then(|a| Ok(a[0] as usize));
/// assert_eq!(fake.read(&mut vec![10]).unwrap(), 10);
/// # }
/// ```
/// # Attribute arguments
///
/// ## path
///
/// Indicates that the struct came from an imported module.
///
/// ### Examples
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
///         // the type is being imported
///         // so we have to tell faux where it came from
///         use super::MyStruct;
///
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
///     // the module is being imported
///     // so we have to tell faux where it came from
///     use crate::foo;
///
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
/// faux::when!(x.three).then_return(30);
/// faux::when!(x.four).then_return(40);
/// faux::when!(x.five).then_return(50);
///
/// assert_eq!(x.three(), 30);
/// assert_eq!(x.four(), 40);
/// assert_eq!(x.five(), 50);
/// # }
/// ```
///
/// ## self_type
///
/// Tells the attribute how real instances of the mockable struct are
/// being stored by [`#[create]`](create). Read the [docs in
/// create](create#self_type) for more information.
///
/// Because this argument specifies how the struct is stored, every
/// mockable method must use a [receiver] that can be obtained from
/// the specified `self_type`. For example, if `self_type = Rc`, then
/// a `&self` or an `self: Rc<Self>` can be used as receivers, but a
/// `&mut self` cannot. If you need an unsupported combination, please
/// file an issue.
///
/// ### Examples
///
/// ```
/// use std::rc::Rc;
///
/// #[faux::create(self_type = "Rc")]
/// pub struct ByRc {}
///
/// #[faux::methods(self_type = "Rc")]
/// impl ByRc {
///     // you can return an owned instance
///     pub fn new() -> Self {
///         ByRc {}
///     }
///
///     // or an instance wrapped in the `self_type`
///     pub fn new_rc() -> Rc<Self> {
///         Rc::new(ByRc {})
///     }
///
///     // methods with an Rc<Self> receiver type can be called
///     pub fn by_rc(self: Rc<Self>) {}
///
///     // Rc<Self> derefs to &self, so this also works
///     pub fn by_ref(&self) {}
/// }
/// # fn main() {}
/// ```
///
/// ### Allowed values:
/// * `#[methods(self_type = "Owned")]` (default)
/// * `#[methods(self_type = "Rc")]`
/// * `#[methods(self_type = "Arc")]`
/// * `#[methods(self_type = "Box")]`
///
/// Note that methods with a `self: Pin<...>` receiver are
/// mockable. `self_type` does not specify what types of receivers can
/// be mocked, but how `faux` stores the instances internally.
///
/// # Panics
///
/// ## Non-mocked methods
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
/// Spies (real instances with mocked methods) are not supported.
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
/// ## Unsupported receiver/`self_type` combinations
///
/// If a mockable method uses `self: Rc<Self>` as a receiver and the
/// `self_type` is not `Rc`, `faux` performs a uniqueness check to
/// prevent unsound behavior. This also applies to `self: Arc<Self>`.
///
/// In the case of a panic, the message `faux` produces guides you
/// towards using the `self_type` argument in the `faux` attributes.
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
/// let rcd = Rc::new(Owned::new());
/// // this works because this is the only Rc for the instance
/// rcd.by_rc();
///
/// let rcd = Rc::new(Owned::new());
/// // clone the Rc so the uniqueness check fails
/// let clone = rcd.clone();
/// rcd.by_rc(); // <~ panics: reference is not unique
/// # }
/// ```
///
/// # Known Limitations
///
/// * [#13]: Within a module, for a single struct, only a single inherent `impl`
/// and a single trait `impl` per trait may exist.
/// * [#14]: Methods cannot have arguments of the same type as their struct.
/// * [#18]: Generic methods and `impl` return types are not supported.
///
/// [#13]: https://github.com/nrxus/faux/issues/13
/// [#14]: https://github.com/nrxus/faux/issues/14
/// [#18]: https://github.com/nrxus/faux/issues/18
///
/// # Caveats
///
/// ## Returning mockable struct
///
/// Returning the mockable struct wrapped as a generic of another type
/// (e.g., `Option<Self>`) is not currently supported. The exception
/// to this is returning an instance wrapped by the
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
/// A workaround is to place these functions in an untagged `impl`
/// block and have them call methods inside the tagged `impl`.
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
/// `#[methods]` can be added to blocks of the form `impl
/// path::to::Type` as long as the path does not contain the `super`
/// or `crate` keywords. If it does, use the [`path`](#path) argument to
/// explicitly specify the path.
///
/// [receiver]: https://doc.rust-lang.org/reference/items/associated-items.html#methods
pub use faux_macros::methods;

/// Creates a [`When`] instance to mock a specific method in a struct.
///
/// Callers may specify argument matchers to limit the arguments for
/// which the method is mocked. Matchers can only be specified if all
/// arguments implement [`Debug`](std::fmt::Debug). The debug message
/// is printed if any of the arguments fail to match.
///
/// The method to mock must be be in an `impl` blocked tagged by
/// [`#[methods]`](methods).
///
/// # Examples
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
///
///     // specify all arguments
///     faux::when!(mock.some_method(8, 9)).then_return(10);
///     // actual method calls have to match expectations
///     assert_eq!(mock.some_method(8, 9), 10);
///     // mock.some_method(1, 1); // <~~ panics - arguments do not match
///
///     // check only the second argument
///     faux::when!(mock.some_method(_, 4)).then_return(20);
///     // only the second argument is being matched against
///     // so the first argument could be anything
///     assert_eq!(mock.some_method(999, 4), 20);
///     assert_eq!(mock.some_method(123, 4), 20);
///     // mock.some_method(999, 3); // <~~ panics - second argument does not match
///
///     // no argument matchers
///     faux::when!(mock.some_method).then_return(3);
///     // the arguments do not matter at all
///     assert_eq!(mock.some_method(1337, 20), 3);
///     assert_eq!(mock.some_method(4, 5), 3);
///     assert_eq!(mock.some_method(7, 6), 3);
/// }
/// ```
///
/// An argument mismatch would look something like:
///
/// ```term
/// thread 'main' panicked at 'failed to call mock on 'Foo::some_method':
/// ✗ Arguments did not match
///   Expected: [8, 9]
///   Actual:   [1, 1]
///
///   Argument 0:
///     Expected: 8
///     Actual:   1
///   Argument 1:
///     Expected: 9
///     Actual:   1
/// ```
///
/// # Argument Matchers
///
/// Argument matchers are specified by passing them to `when!`:
///
/// ```
/// # #[faux::create]
/// # pub struct Foo;
/// # #[faux::methods]
/// # impl Foo {
/// #     pub fn my_method(&self) -> i32 {
/// #        panic!()
/// #    }
/// # }
/// # fn main() {
/// # let mut my_struct = Foo::faux();
/// faux::when!(my_struct.my_method(/* matchers here */));
/// # }
/// ```
///
/// This rougly translates to:
///
/// ```
/// # #[faux::create]
/// # pub struct Foo;
/// # #[faux::methods]
/// # impl Foo {
/// #     pub fn my_method(&self) -> i32 {
/// #        panic!()
/// #    }
/// # }
/// # fn main() {
/// # let mut my_struct = Foo::faux();
/// faux::when!(my_struct.my_method).with_args((/* matchers here */));
/// # }
/// ```
///
/// ### Matcher syntax
///
/// To make argument matching easy to use, `when!` provides some
/// syntactic sugar that converts given arguments to the appropiate
/// [`ArgMatcher`] and passes them to [`with_args`]. If this proves
/// difficult in your use case, you can use [`with_args`] directly.
///
/// Each of the following specify an equivalent [`ArgMatcher`] for a
/// single argument:
///
/// | `when!` arg     | [`ArgMatcher`] |
/// |-----------------|------------------------|
/// | `{expr}`        | [`eq({expr})`]         |
/// | `_`             | [`any()`]              |
/// | `_ == {expr}`   | [`eq_against({expr})`] |
/// | `_ = {matcher}` | [`{matcher}`]          |
///
/// Replace `_` with `*_` in the last two rows to match against
/// references. More specifically, this converts the matcher from
/// `ArgMatcher<T>` into `ArgMatcher<&T>` using [`into_ref_matcher`].
///
/// ### Examples
///
/// ```
/// #[faux::create]
/// pub struct MyStruct;
/// #[faux::methods]
/// impl MyStruct {
///     pub fn my_method(&self, a: &i32, b: i32) -> i32 {
///        panic!()
///    }
/// }
///
/// # fn main() {
/// let mut my_struct = MyStruct::faux();
///
/// // the eq matcher works even though the first argument is a reference
/// // the `_` matcher will match any argument
/// faux::when!(my_struct.my_method(3, _)).then_return(4);
/// assert_eq!(my_struct.my_method(&3, 20), 4);
///
/// // a type that implements `PartialEq<i32>` but is not an `i32`
/// #[derive(Debug)]
/// struct OtherNumber(i64);
///
/// impl PartialEq<i32> for OtherNumber {
///     fn eq(&self, rhs: &i32) -> bool {
///         self.0 == *rhs as i64
///     }
/// }
///
/// // `_ == {expr}` to test equality of different types
/// // `*_ == {expr}` to dereference an argument before matching
/// faux::when!(my_struct.my_method(
///     *_ == OtherNumber(5),
///     _ == OtherNumber(20),
/// )).then_return(8);
/// assert_eq!(my_struct.my_method(&5, 20), 8);
///
/// // `_ = {matcher}` will pass the matcher to `with_args` as written
/// // `*_ = {matcher}` will match against a dereferenced argument
/// faux::when!(my_struct.my_method(
///     *_ = faux::matcher::eq_against(OtherNumber(4)),
///     _ = faux::matcher::eq(9),
/// )).then_return(20);
/// assert_eq!(my_struct.my_method(&4, 9), 20);
///
/// // pattern! and from_fn! are allowed just as any other matcher
/// faux::when!(my_struct.my_method(
///     *_ = faux::pattern!(10..=20),
///     _ = faux::from_fn!(|arg: &i32| *arg > 50),
/// )).then_return(80);
/// assert_eq!(my_struct.my_method(&11, 60), 80);
/// # }
///
/// ```
///
/// [`When`]: struct.When.html
/// [`any()`]: matcher/fn.any.html
/// [`eq_against({expr})`]: matcher/fn.eq_against.html
/// [`ArgMatcher`]: matcher/trait.ArgMatcher.html
/// [`{matcher}`]: matcher/trait.ArgMatcher.html
/// [`into_ref_matcher`]: matcher/trait.ArgMatcher.html#method.into_ref_matcher
/// [`eq({expr})`]: matcher/fn.eq.html
/// [`with_args`]: struct.When.html#method.with_args
pub use faux_macros::when;

#[doc(inline)]
pub use when::When;

#[doc(inline)]
pub use matcher::ArgMatcher;

// exported so generated code can call for it
// but purposefully not documented
pub use mock_store::{LazyMethodId, MaybeFaux, MockStore};

#[doc(include = "../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
