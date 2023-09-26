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
//! * [`#[create]`](create): transforms a struct into a mockable
//! equivalent
//! * [`#[methods]`](methods): transforms the methods in an `impl`
//! block into their mockable equivalents
//! * [`when!`]: initializes a method stub by returning a
//! [`When`]. Passing optional argument matchers restricts which
//! arguments will invoke the stub.
//! * [`When`]: lets you stub a method's return value or
//! implementation
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
//! # pub trait Serialize {}
//! # pub trait Deserialize {}
//! # #[derive(Debug, Clone, PartialEq)]
//! # pub struct MyData { id: String }
//! # #[derive(Clone, Debug, PartialEq)]
//! # pub struct MyResponse { name: String }
//! # impl Serialize for MyData {}
//! # impl Deserialize for MyResponse {}
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
//!     pub fn get(&self, path: &str, headers: &Headers) -> String {
//!         /* makes network calls that we'd rather not do in unit tests */
//!         # unreachable!()
//!     }
//!
//!     pub fn post<B: Serialize, T: Deserialize>(&self, path: &str, body: &B) -> T {
//!         /* makes network calls that we'd rather not do in unit tests */
//!         # unreachable!()
//!     }
//! }
//!
//! #[cfg(test)]
//! #[test]
//! fn test() {
//! # }
//! # fn main() {
//!   // use the generated `faux()` function to create a mock instance
//!   let mut mock = HttpClient::faux();
//!
//!   let headers = Headers { authorization: "Bearer foobar".to_string() };
//!
//!   // use `faux::when!` to stub the behavior of your methods
//!   // you can specify arguments to match against when the stub is invoked
//!   faux::when!(
//!       // arguments are converted into argument matchers
//!       // the default argument matcher performs an equality check
//!       // use `_` to create a universal argument matcher
//!       // the argument matchers below specify to ignore the first argument
//!       // but that the second one must equal `headers`
//!       mock.get(_, headers.clone())
//!   )
//!   // stub the return value
//!   .then_return("{}".to_string());
//!
//!   assert_eq!(mock.get("any/path/does/not/mater", &headers), "{}");
//!   assert_eq!(mock.get("as/i/said/does/not/matter", &headers), "{}");
//!
//!   // if you want to stub all calls to a method, you can omit argument matchers
//!   faux::when!(mock.get).then_return("OK".to_string());
//!   let other_headers = Headers { authorization: "other-token".to_string() };
//!   assert_eq!(mock.get("other/path", &other_headers), "OK");
//!
//!   // `faux` allows mocking generic methods but will
//!   // generally require them to be explicitely named
//!   let data = MyData { id: "my-id".to_owned() };
//!   let expected_respone = MyResponse { name: "my name".to_owned() };
//!   faux::when!(mock.post::<MyData, _>(_, data.clone())).then_return(expected_respone.clone());
//!   assert_eq!(mock.post::<_, MyResponse>("/some/post/path", &data), expected_respone);
//! }

//! ```
//!
//! ## Stubbing the same method multiple times
//!
//! A single method can be stubbed multiple times. When doing so,
//! `faux` checks every stub for the method in a last-in-first-out
//! fashion until it finds a stub whose argument matchers match the
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
//!   // catch-all stub to return "OK"
//!   faux::when!(mock.post).then_return("OK".to_string());
//!   // stub for specific headers to return "{}"
//!   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//!
//!   assert_eq!(mock.post("some/path", &headers), "{}"); // matches specific stub
//!   assert_eq!(mock.post("some/path", &other_headers), "OK"); // matches catch-all stub
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
//! ## Stubbing implementation
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
//! ## Stubbing with non-static data
//!
//! Let's add a new method to our `HttpClient` that returns borrowed
//! data. This cannot be stubbed using safe code, so `faux` provides
//! `.then_unchecked()` and `.then_unchecked_return()` to stub such
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
//!   // they allow stubbing methods that return non-static values (e.g. references)
//!   // or to stub using non-static closures
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
//! `faux` lets you stub the return value or implementation of:
//!
//! * Async methods
//! * Trait methods
//! * Generic struct methods
//! * Generics methods
//! * Methods with pointer self types (e.g., `self: Rc<Self>`)
//! * Methods in external modules
//! * Support for `Debug`, `Default`, `Clone`, `Send`, and `Sync`
//! derive/auto traits.
//!
//! `faux` also provides easy-to-use argument matchers.
//!
//! [mocks]: https://martinfowler.com/articles/mocksArentStubs.html

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
/// ## Non-stubbed methods
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
/// // fake.get is not stubbed
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
/// * [#14]: Methods cannot have arguments of the same type as their struct.
/// * [#18]: `impl` return types are not supported.
///
/// [#14]: https://github.com/nrxus/faux/issues/14
/// [#18]: https://github.com/nrxus/faux/issues/18
///
/// # Caveats
///
/// ## Returning mockable struct
///
/// When referring to the mockable struct in the signature (either by
/// name or by `Self`) only special cases are allowed. In particular,
/// it is only allowed in the return position of the signature for the
/// follow cases:
///
/// * Returning the struct itself (e.g., `fn new() -> Self`)
///
/// * Returning the struct wrapped directly in: `Rc`, `Arc`, `Box`,
/// `Result`, or `Option`. For `Result`, referring to the struct is
/// only allowed if it's the `Ok` variant of the result. (e.g., `fn
/// load() -> Result<Self, Error>`)
///
/// Any other kind of return type that refers to the mocked struct is
/// not supported by `faux`. Please file an issue if you have a use
/// case that you believe should be common enough for `faux` to handle
/// automatically.
///
/// A workaround is to place the functions in an untagged `impl` block
/// and have them call methods inside the tagged `impl`.
///
/// ```
/// pub enum Either<X, Y> {
///     Left(X),
///     Right(Y),
/// }
///
/// #[faux::create]
/// pub struct MyStruct {
///     x: i32
/// }
///
/// #[faux::methods]
/// impl MyStruct {
///     fn new() -> Self {
///         MyStruct { x: 4 }
///     }
/// }
///
/// // do not tag this one
/// impl MyStruct {
///     pub fn make_either() -> Either<Self, String> {
///         Either::Left(MyStruct::new())
///     }
/// }
///
/// # fn main() {
/// let x = MyStruct::make_either();
/// assert!(matches!(x, Either::Left(MyStruct { .. })));
/// # }
/// ```
///
/// ## Paths in types
///
/// `#[methods]` can be added to blocks of the form `impl
/// path::to::Type` as long as the path does not contain the `super`
/// or `crate` keywords. If it does, use the [`path`](#path) argument
/// to explicitly specify the path.
///
/// [receiver]: https://doc.rust-lang.org/reference/items/associated-items.html#methods
pub use faux_macros::methods;

/// Creates a [`When`] instance to stub a specific method in a struct.
///
/// Callers may specify argument matchers to limit the arguments for
/// which the method is stubbed. Matchers can only be specified if all
/// arguments implement [`Debug`](std::fmt::Debug). The debug message
/// is printed if any of the arguments fail to match.
///
/// The method to stub must be be in an `impl` blocked tagged by
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
/// thread 'main' panicked at 'failed to call stub on 'Foo::some_method':
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
/// # Generic methods
///
/// Generic methods are mocked by specifying the generic arguments as
/// part of the `when!` expression. There is currently no way to
/// generically mock a method for all possible types, each type must
/// be mocked independently.
///
/// ## Examples
///
/// ```
/// # #[faux::create]
/// # pub struct Foo;
/// #[faux::methods]
/// impl Foo {
///     pub fn gen_method<T>(&self, t: T) -> i32 {
///         /* snip */
/// #        panic!()
///     }
/// }
///
/// # fn main() {
/// # let mut my_struct = Foo::faux();
///
/// // the following line would not compile; we have to tell it
/// // what the generic argument needs to be for this mock
/// // faux::when!(my_struct.gen_method).then_return(4);
///
/// // the following line would not compile;
/// // it is not a valid expression by Rust rules
/// // faux::when!(my_struct.gen_method::<i32>).then_return(4);
///
/// // a valid expression so we can tell it that any i32 will return 4
/// faux::when!(my_struct.gen_method::<i32>(_)).then_return(4);
/// // different types are different mocks: any string will return 2
/// faux::when!(my_struct.gen_method::<&str>(_)).then_return(2);
///
/// assert_eq!(my_struct.gen_method(-1), 4);
/// assert_eq!(my_struct.gen_method("hello"), 2);
/// # }
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
/// ## Matcher syntax
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

mod mock;

use core::fmt;
use std::fmt::Formatter;
use std::sync::Arc;

/// What all mockable structs get transformed into.
///
/// Either a real instance or a mock store to store/retrieve all the
/// mocks.
///
/// Exposed so generated code can use it for it but purposefully not
/// documented. Its definition is an implementation detail and thus
/// not meant to be relied upon.
///
/// ```
/// fn implements_sync<T: Sync>(_: T) {}
///
/// implements_sync(3);
/// implements_sync(faux::MaybeFaux::Real(3));
/// ```
///
/// ```
/// fn implements_debug<T: std::fmt::Debug>(_: T) {}
///
/// implements_debug(3);
/// implements_debug(faux::MaybeFaux::Real(3));
/// ```
///
/// ```
/// fn implements_default<T: Default>(_: T) {}
///
/// implements_default(3);
/// implements_default(faux::MaybeFaux::Real(3));
/// ```
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum MaybeFaux<T> {
    Real(T),
    Faux(Faux),
}

impl<T: Default> Default for MaybeFaux<T> {
    fn default() -> Self {
        MaybeFaux::Real(T::default())
    }
}

impl<T> MaybeFaux<T> {
    pub fn faux(name: &'static str) -> Self {
        MaybeFaux::Faux(Faux::new(name))
    }
}

/// The internal representation of a mock object
///
/// Exposed so generated code can use it but purposefully not
/// documented. Its mere existence is an implementation detail and not
/// meant to be relied upon.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct Faux {
    store: Arc<mock::Store<'static>>,
}

impl Faux {
    pub fn new(name: &'static str) -> Self {
        Faux {
            store: Arc::new(mock::Store::new(name)),
        }
    }

    /// Return a mutable reference to its internal mock store
    ///
    /// Returns `None` if the store is being shared by multiple mock
    /// instances. This occurs when cloning a mock instance.
    pub(crate) fn unique_store(&mut self) -> Option<&mut mock::Store<'static>> {
        Arc::get_mut(&mut self.store)
    }

    #[doc(hidden)]
    /// Attempt to call a stub for a given function and input.
    ///
    /// Stubs are attempted in the reverse order of how they were
    /// inserted. Namely, the last inserted stub will be attempted
    /// first. The first stub for whom the input passes its invocation
    /// matcher will be activated and its output returned. If one
    /// cannot be found an error is returned.
    ///
    /// # Safety
    ///
    /// Do *NOT* call this function directly.
    /// This should only be called by the generated code from #[faux::methods]
    pub unsafe fn call_stub<R, I, O>(
        &self,
        id: fn(R, I) -> O,
        fn_name: &'static str,
        input: I,
    ) -> Result<O, InvocationError> {
        let mock = self.store.get(id, fn_name)?;
        mock.call(input).map_err(|stub_error| InvocationError {
            fn_name: mock.name(),
            struct_name: self.store.struct_name,
            stub_error,
        })
    }
}

pub struct InvocationError {
    struct_name: &'static str,
    fn_name: &'static str,
    stub_error: mock::InvocationError,
}

impl fmt::Display for InvocationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.stub_error {
            mock::InvocationError::NeverStubbed => write!(
                f,
                "`{}::{}` was called but never stubbed",
                self.struct_name, self.fn_name
            ),
            mock::InvocationError::Stub(errors) => {
                writeln!(
                    f,
                    "`{}::{}` had no suitable stubs. Existing stubs failed because:",
                    self.struct_name, self.fn_name
                )?;
                let mut errors = errors.iter();
                if let Some(e) = errors.next() {
                    f.write_str("✗ ")?;
                    fmt::Display::fmt(e, f)?;
                }
                errors.try_for_each(|e| {
                    f.write_str("\n\n✗ ")?;
                    fmt::Display::fmt(e, f)
                })
            }
        }
    }
}

#[cfg(doc)]
mod readme_tests;
