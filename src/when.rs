use crate::MockStore;
use std::any::TypeId;

/// Stores who and what to mock and provides methods to mock the
/// method both safely and unsafely.
///
/// See [when!] for how to get a instance of this
/// struct.
///
/// [when!]: macro.when.html
pub struct When<'q, I, O> {
    id: TypeId,
    store: &'q mut MockStore,
    // *const for variance -- I think that's what I want.
    _marker: std::marker::PhantomData<(*const I, *const O)>,
}

impl<'q, I, O> When<'q, I, O> {
    #[doc(hidden)]
    pub fn new(id: TypeId, store: &'q mut MockStore) -> Self {
        When {
            id,
            store,
            _marker: std::marker::PhantomData,
        }
    }

    /// Mocks the method stored in the `When` with the given closure
    /// for the saved instance. This mock has no restrictions on the
    /// lifetimes for the inputs, outputs, nor the mocked function
    /// itself for maximum flexibility.
    ///
    /// The input for the given closure is a tuple of all its
    /// non-receiver parameters (not `self`, `&self`, nor `&mut
    /// self`). While this method is "type" safe for the types in the
    /// mocked method, it is not lifetime safe. See [safety].
    ///
    /// [safety]: #safety
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn no_args(&mut self) -> &i32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    ///
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    ///
    ///     pub fn multi_args(self, a: &i32, b: i8) -> u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // the output can be a reference to the environment
    ///   // but this can be *very* dangerous so be careful
    ///   // note that the closure still has an argument, it is just an empty tuple
    ///   let x = 5;
    ///   unsafe { faux::when!(mock.no_args).then(|_empty: ()| &x) }
    ///   assert_eq!(*mock.no_args(), 5);
    ///
    ///   // unit tuples do not need parentheses
    ///   unsafe { faux::when!(mock.single_arg).then(|input| vec![input as i8]) }
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///
    ///   // inputs can be references
    ///   unsafe { faux::when!(mock.multi_args).then(|(&a, b)| a as u32 + b as u32) }
    ///   assert_eq!(mock.multi_args(&5, 23), 28)
    /// }
    ///
    /// ```
    ///
    ///
    /// # Safety
    ///
    /// This function effectively erases the lifetime relationships of
    /// the inputs and outputs. It is the user's responsability to not
    /// pass a mock that would capture a variable that would be used
    /// after it has been deallocated.
    ///
    /// Another way in which this function is unsafe is if the output
    /// of this function has a logical lifetime link to the input.  At
    /// the moment the mock gets called, that link would be erased
    /// which could create multiple mutable references to the same
    /// object.
    ///
    /// Example:
    ///
    /// ```
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn out_ref(&self, a : &mut i32) -> &mut i32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///   // set up the mock such that the output is the same reference as the input
    ///   unsafe { faux::when!(mock.out_ref).then(|i| i) }
    ///
    ///   let mut x = 5;
    ///   // y is now a mutable reference back x
    ///   // but there is no compile-time link between the two
    ///   let y = mock.out_ref(&mut x);
    ///
    ///   // We can check that they are both the same value
    ///   assert_eq!(*y, 5);
    ///   assert_eq!(x, 5);
    ///
    ///   // x now changes y. This is UB and is not allowed in safe Rust!
    ///   x += 1;
    ///   assert_eq!(x, 6);
    ///   assert_eq!(*y, 6);
    ///
    ///   // and if we change y then x also gets changed
    ///   *y += 1;
    ///   assert_eq!(x, 7);
    ///   assert_eq!(*y, 7);
    /// }
    /// ```
    pub unsafe fn then(self, mock: impl FnOnce(I) -> O + Send) {
        self.store.unsafe_mock_once(self.id, mock);
    }

    /// Mocks the method stored in the `When` with the given closure
    /// for the saved instance. This mock is restricted only to static
    /// inputs, outputs, and closures. While this is very restrictive
    /// it allows for a purely safe interface. See [then] for the
    /// unsafe version.
    ///
    /// The input for the given closure is a tuple of all its
    /// non-receiver parameters (not `self`, `&self`, nor `&mut
    /// self`).
    ///
    /// [then]: #method.then
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn no_args(&mut self) -> i32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    ///
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    ///
    ///     pub fn multi_args(self, a: &i32, b: i8) -> u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // closure still has an argument, it is just an empty tuple
    ///   faux::when!(mock.no_args).safe_then(|_empty: ()| 5);
    ///   assert_eq!(mock.no_args(), 5);
    ///
    ///   // unit tuples do not need parentheses
    ///   faux::when!(mock.single_arg).safe_then(|input| vec![input as i8]);
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///
    ///   // one of the arguments for `multi_args` is not a static type
    ///   // so the following line would not compile
    ///   // faux::when!(mock.multi_args).safe_then(|(_, _)| 5);
    /// }
    ///
    /// ```
    pub fn safe_then(self, mock: impl FnOnce(I) -> O + 'static + Send)
    where
        I: 'static,
        O: 'static,
    {
        self.store.mock_once(self.id, mock);
    }
}
