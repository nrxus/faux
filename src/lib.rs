#![cfg_attr(doctest, feature(external_doc))]
#![allow(clippy::needless_doctest_main)]

//! A library to create [mocks] out of `struct`s.
//!
//! `faux` allows you to mock the behavior (i.e. methods) of structs
//! without complicating or polluting your code. This is useful during
//! testing to stub the behavior of a hard to test struct and focus on
//! the testable parts of your code.
//!
//! At a high level `faux` is split into:
//!
//! * [`#[create]`](create): Morphs a struct into a mockable equivalent
//! * [`#[methods]`](methods): Morphs the methods in an impl block into
//! their mockable equivalent
//! * [`when!`]: Initializes the stubbing of a specific method; either
//! with or without argument matchers
//! * [`When`]: Configures the stub by setting either a stub value or
//! implementation
//!
//! Visit their individual docs for further details on how to use them
//! to help you mock.
//!
//! # Getting Started
//!
//! `faux` makes liberal use of unsafe rust features, and it is only
//! recommended for use inside tests. To prevent `faux` from leaking
//! into your production code, set it as a `dev-dependency` in you
//! `Cargo.toml`
//!
//! ```toml
//! [dev-dependencies]
//! faux = "0.0.8"
//! ```
//!
//! # Examples
//!
//! ```
//! // restrict faux to tests by using #[cfg_attr(test, ...)]
//! // faux::create marks a struct as mockable
//! // this includes generating a `faux()` associated function
//! // `HttpClient::faux()` will create a mock instance
//! #[cfg_attr(test, faux::create)]
//! # #[faux::create]
//! pub struct HttpClient { /* */ }
//!
//! // this is just a bag of data with no behavior
//! // so we do not mark it as mockable
//! #[derive(PartialEq, Clone, Debug)]
//! pub struct Headers {
//!     pub authorization: String,
//! }
//!
//! // faux::methods marks every public method in the impl as mockable
//! #[cfg_attr(test, faux::methods)]
//! # #[faux::methods]
//! impl HttpClient {
//!     pub fn post(&self, path: &str, headers: &Headers) -> String {
//!         /* does network calls that we rather not do in unit tests */
//!         # unreachable!()
//!     }
//!
//!     pub fn host(&self) -> &str {
//!         /* returns a reference to some internal data */
//!         # unreachable!()
//!     }
//! }
//!
//! #[cfg(test)]
//! #[test]
//! fn test() {
//!   // Use the generated `faux` associated function to create a mock instance
//!   let mut mock = HttpClient::faux();
//!
//!   let headers = Headers { authorization: "Bearer foobar".to_string() };
//!
//!   // use faux::when! to mock the behavior of your methods
//!   // you can use `_` if you do not care about specific arguments
//!   // or pass an argument that implements `PartialEq` to compare against
//!   // we can then mock the return stub using `.then_return(...)`
//!   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//!   // we setup the mock for any path, but only specific headers to let's pass those
//!   assert_eq!(mock.post("any/path/does/not/mater", &headers), "{}");
//!
//!   // faux::when! also accepts getting no argument matchers
//!   faux::when!(mock.post).then_return("OK".to_string());
//!   // we can now pass different headers since our mock matches everything
//!   assert_eq!(
//!       mock.post(
//!           "some/other/path",
//!           &Headers { authorization: "other-token".to_string() }
//!       ),
//!       "OK".to_string()
//!   );
//!
//!   // for implementation mocking: use `.then(...)`
//!   faux::when!(mock.post).then(|(path, _)| path.to_string());
//!   assert_eq!(mock.post("another/path", &headers), "another/path");
//!
//!   // unsafe versions of `.then` and `.then_return` exist to mock
//!   // methods that return non-static values (e.g., references)
//!   // or to mock using non-static closures
//!   let ret = "some-value".to_string();
//!   unsafe { faux::when!(mock.host).then_unchecked_return(ret.as_str()) }
//!   assert_eq!(mock.host(), &ret);
//! }
//! #
//! # fn main() {
//! #   // Use the generated `faux` associated method to create a mock instance
//! #   let mut mock = HttpClient::faux();
//! #
//! #   // setup what the value should return based on some arguments
//! #   let headers = Headers { authorization: "Bearer foobar".to_string() };
//! #   // mock it for *any* path, but only headers equal to the expected
//! #   faux::when!(mock.post(_, headers.clone())).then_return("{}".to_string());
//! #   assert_eq!(mock.post("any/path/does/not/mater", &headers), "{}");
//! #
//! #   // If you do not care about any arguments, don't specify them at all
//! #   faux::when!(mock.post).then_return("OK".to_string());
//! #   assert_eq!(
//! #       mock.post(
//! #           "some/other/path",
//! #           &Headers { authorization: "other-token".to_string() }
//! #       ),
//! #       "OK".to_string()
//! #   );
//! #
//! #   // You can have full control over the mock implementation not just the return value
//! #   faux::when!(mock.post).then(|(path, _)| path.to_string());
//! #   assert_eq!(mock.post("another/path", &headers), "another/path");
//! #
//! #   // an unsafe version exist to mock methods with non-static outputs
//! #   // or non-static mock closures
//! #   let ret = "some-value".to_string();
//! #   unsafe { faux::when!(mock.host).then_unchecked_return(ret.as_str()) }
//! #   assert_eq!(mock.host(), &ret);
//! #  }
//! ```
//!
//! # Features
//!
//! * Argument matchers
//! * Return value mocking
//! * Implementation mocking
//! * Async methods
//! * Trait methods
//! * Generic struct methods
//! * Arbitrary self types (e.g., `self: Rc<Self>`)
//! * External modules
//!
//! [mocks]: https://martinfowler.com/articles/mocksArentStubs.html

mod mock;
mod mock_store;

pub mod matcher;
pub mod when;

/// Transforms a struct into a mockable version of itself.
///
/// It creates an associated function for the tagged struct called
/// `faux` and masks the original definition of the struct by changing
/// its name.
///
/// Use [`cargo-expand`] to see what your struct expands to after the
/// macro.
///
/// # Requirements
///
/// This macro deliberately fails to compile if any of the struct's
/// fields are not private. Otherwise, a user of the struct could try
/// to access the field directly when it no longer exists in the
/// transformed version.
///
/// Only methods within `impl` blocks tagged by
/// [`#[methods]`](methods) may use any of the struct fields.
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
/// // creates a mock out of MyStruct
/// let my_mock = MyStruct::faux();
/// # }
/// ```
///
/// # Attribute arguments
///
/// ## self_type
///
/// Customize how the real instances of the mockable struct are
/// stored.
///
/// When set, all of the `impl` blocks tagged with
/// [`#[methods]`](methods) must specify the same `self_type`.
///
/// ### Explanation
///
/// By default `#[faux::create]` transform a struct from:
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
/// This works great when the creation of `MyStruct` returns an owned
/// instance of it. There are some cases, however, when the only way
/// to create an object returns a smart pointer to the object (i.e.,
/// `Rc<Self>`). This attribute argument let's you customize how
/// `faux` will wrap the real instance of your object to support such
/// cases.
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
/// * `#[create(self_type = "Rc")]`
/// * `#[create(self_type = "Arc")]`
/// * `#[create(self_type = "Box")]`
/// * `#[create(self_type = "Owned")]`
///   * this is the default and not necessary
///
/// [`cargo-expand`]: https://github.com/dtolnay/cargo-expand
///
pub use faux_macros::create;

/// Transforms methods in an `impl` block into mockable versions of
/// themselves.
///
/// The mockable methods can then be mocked using [`when!`].
///
/// Associated functions and private methods are not mocked, and are
/// instead proxied to the real implementation.
///
/// # Requirements
///
/// [`#[create]`](create) must have been previously called for this
/// struct.
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
/// // mocks every public method in this inherent impl block
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
/// // mocks every method in this trait impl block
/// #[cfg_attr(test, faux::methods)]
/// # #[faux::methods]
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
/// Tells the attribute how real instances are being stored by
/// [`#[create]`](create). Read the [docs in create](create#self_type)
/// for more information.
///
/// Because this argumnet specifies how the struct is stored, every
/// mockable method must use a [receiver] that can be obtained from
/// the specified `self_type`. For example, if `self_type = Rc`, then
/// a `&self` or an `self: Rc<Self>` can be used as receivers but a
/// `&mut self` cannot. If a certain combination of specified
/// `self_type` and method receiver should be doable but not working,
/// please file an issue.
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
/// ### Allowed values:
/// * `#[methods(self_type = "Rc")]`
/// * `#[methods(self_type = "Arc")]`
/// * `#[methods(self_type = "Box")]`
/// * `#[methods(self_type = "Owned")]`
///   * this is the default and not necessary
///
/// While `Pin<...>` is not allowed as a `self_type` we can still mock
/// methods with `self: Pin<...>`. This argument does not specify what
/// receivers can be mocked but how `faux` stores the instances
/// internally.
///
/// # Panics
///
/// ## Non-mocked methods
///
/// `faux` will not return a default value on non-mocked methods so it
/// panics instead.
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
/// ## Receiver/`self_type` combinations
///
/// If a mockable method uses `self: Rc<Self>` as a receiver and the
/// self type is not `Rc`, a check is done to prevent unsoundness when
/// converting the owned instance into an `Rc<Self>`. This check
/// guarantees that there are no other `Rc<Self>`s, and panics if
/// there are. This also applies to `self: Arc<Self>`
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
/// In the case of a panic, the message `faux` produces guides you
/// towards using the `self_type` argument in the faux attributes
///
/// # Known Limitations
///
/// * [#13]: Within a module, only a single inherent impl and a single
/// trait impl per trait per type may exist.
/// * [#14]: Methods of a given struct cannot have arguments of that
/// struct as parameters.
/// * [#18]: Generic methods and `impl` return types are not allowed
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
///
/// [receiver]: https://doc.rust-lang.org/reference/items/associated-items.html#methods
pub use faux_macros::methods;

/// Creates a [`When`] instance to mock a specific method in a struct.
///
/// `when!` can optionally specify argument matchers. If specified,
/// the method will only be mocked when all the argument matchers are
/// met.
///
/// The method to mock must be be in an `impl` blocked tagged by
/// [`#[methods]`](methods)
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
///     // actual arguments have to match the expected above
///     assert_eq!(mock.some_method(8, 9), 10);
///     // mock.some_method(1, 1) <~~ panics - arguments do not match
///
///     // check only the second argument
///     faux::when!(mock.some_method(_, 4)).then_return(20);
///     // only the second argument is being matched against
///     // so the first argument could be anything
///     assert_eq!(mock.some_method(999, 4), 20);
///     assert_eq!(mock.some_method(123, 4), 20);
///     // mock.some_method(999, 3) <~~ panics - second argument does not match
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
/// # Argument Matchers
///
/// Argument matchers are specified by passing them in the `when!`
/// statement:
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
/// To make argument matching easy to use, `when!` has a minimal
/// domain specific language (DSL) that converts given arguments to
/// the appropiate [`ArgMatcher`] and passes them to [`with_args`]. If
/// this proves difficult in your use case you may always default back
/// to using [`with_args`] directly.
///
/// ### DSL
///
/// Each of these specify a matcher for a single argument:
///
/// | `when!` arg     | [`ArgMatcher`]         |
/// |-----------------|------------------------|
/// | `{expr}`        | [`eq({expr})`]         |
/// | `_`             | [`any()`]              |
/// | `_ == {expr}`   | [`eq_against({expr})`] |
/// | `_ = {matcher}` | [`{matcher}`]          |
///
/// A special syntax exists for matching against a reference
/// argument. You may use `*_ = {matcher}` instead of `_ = {matcher}`,
/// or `*_ == {expr}` instead of `_ == {expr}` to invoke
/// [`into_ref_matcher`]. This converts an `ArgMatcher<T>` into
/// `ArgMatcher<&T>` to match against references.
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

// exported so generated code can call for it
// but purposefully not documented
pub use mock_store::{MaybeFaux, MockStore};

#[doc(include = "../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
