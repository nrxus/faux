use crate::{mock::MockTimes, mock_store::MockStore};

/// Stores who to mock, what to mock, and how many times to mock it.
///
/// Created using [when!].
///
/// By default all methods are mocked indefinitely and the mock
/// closures may not consume captured variables. See the [times] and
/// [once] methods to override these default.
///
/// [when!]: macro.when.html
/// [once]: #method.once
/// [times]: #method.times
pub struct When<'q, I, O> {
    id: &'static str,
    store: &'q mut MockStore,
    // *const for variance -- I think that's what I want.
    _marker: std::marker::PhantomData<(*const I, *const O)>,
    times: MockTimes,
}

impl<'q, I, O> When<'q, I, O> {
    #[doc(hidden)]
    pub fn new(id: &'static str, store: &'q mut MockStore) -> Self {
        When {
            id,
            store,
            times: MockTimes::Always,
            _marker: std::marker::PhantomData,
        }
    }

    /// Saves a mock without any lifetime checks.
    ///
    /// The provided mock has no restrictions on the lifetimes of the
    /// inputs, outputs, nor the mocked function itself for maximum
    /// flexibility. While this method is "type" safe for the types in
    /// the mocked method, it is [not lifetime safe].
    ///
    /// The input for the given closure is a tuple of all its
    /// non-receiver parameters.
    ///
    /// [not lifetime safe]: #safety
    /// [times]: #method.times
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
    ///   let x = 5;
    ///   // methods with no arguments have mocks with empty tuples
    ///   unsafe { faux::when!(mock.no_args).then(|_empty: ()| &x) }
    ///   assert_eq!(*mock.no_args(), 5);
    ///
    ///   // unit tuples do not need parentheses
    ///   unsafe { faux::when!(mock.single_arg).then(|input| vec![input as i8]) }
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///   // mock may be activated multiple times
    ///   assert_eq!(mock.single_arg(8), vec![8]);
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
    /// The lifetimes of the inputs, outputs, and captured variables
    /// are not checked. While this gives the caller maximum
    /// flexibility when mocking it also *not* memory safe when used
    /// incorrectly.
    ///
    /// Captured variables could be used after-freed if the mock was
    /// called after them being dropped. This would create undefined
    /// behavior.
    ///
    /// Relationships between inputs, outputs, and captured variable
    /// lifetimes are lost. This allows for easy violations of Rust's
    /// aliasing checks, creating undefined behavior.
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
    ///   // the output is the same reference as the input
    ///   // the lifetimes of the input and output are thus linked
    ///   unsafe { faux::when!(mock.out_ref).then(|i| i) }
    ///
    ///   let mut x = 5;
    ///   // y (the output) is a mutable reference back to x (the input)
    ///   // but there is no compile-time link between the two
    ///   let y = mock.out_ref(&mut x);
    ///
    ///   // We can check that they are both the same value
    ///   assert_eq!(*y, 5);
    ///   assert_eq!(x, 5);
    ///
    ///   // changes in x are reflected in y.
    ///   // This is UB and is not allowed in safe Rust!
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
    pub unsafe fn then(self, mock: impl FnMut(I) -> O + Send) {
        self.store.unsafe_mock(self.id, mock, self.times);
    }

    /// Saves a mock that does not allow a view into its inputs.
    ///
    /// Because the mock does not receive any inputs, it allows for
    /// safe mocking of methods that have non-static inputs. The
    /// output and the mock itself must still be a static type.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn multi_args(&self, a: &i32, b: i8) -> u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // mock has no inputs
    ///   faux::when!(mock.multi_args).then_do(|| 5);
    ///   assert_eq!(mock.multi_args(&3, 4), 5);
    ///   // mock activates multiple times
    ///   assert_eq!(mock.multi_args(&3, 4), 5);
    /// }
    ///
    /// ```
    pub fn then_do(self, mut mock: impl FnMut() -> O + 'static + Send)
    where
        O: 'static,
    {
        unsafe { self.then(move |_: I| mock()) }
    }

    /// Saves an object as the return value of the mock.
    ///
    /// Requires the object to be cloneable and static.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn multi_args(&mut self, a: &i32, b: i8) -> u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   faux::when!(mock.multi_args).then_return(5);
    ///   assert_eq!(mock.multi_args(&2, 3), 5);
    ///   // mock activates multiple times
    ///   assert_eq!(mock.multi_args(&2, 3), 5);
    /// }
    ///
    /// ```
    pub fn then_return(self, mock: O)
    where
        O: 'static + Send + Clone,
    {
        unsafe { self.then(move |_: I| mock.clone()) }
    }

    /// Saves a mock where all the mock lifetimes are checked.
    ///
    /// The provided mock must be static, and it must be mocking a
    /// method with static inputs and ouputs. While this is very
    /// restrictive it allows for a purely safe interface.
    ///
    /// The input for the given closure is a tuple of all its
    /// non-receiver parameters.
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
    ///   // unit tuples do not need parentheses
    ///   faux::when!(mock.single_arg).safe_then(|input| vec![input as i8]);
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///   // mock is active for an infinite number of times
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///
    ///   // one of the arguments for `multi_args` is not a static type
    ///   // so the following line would not compile
    ///   // faux::when!(mock.multi_args).safe_then(|(_, _)| 5);
    /// }
    ///
    /// ```
    pub fn safe_then(self, mock: impl FnMut(I) -> O + 'static + Send)
    where
        I: 'static,
        O: 'static,
    {
        self.store.mock(self.id, mock, self.times);
    }

    /// Limits the number of times a mock is active.
    ///
    /// Calls past the limit results in a panic.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // limit to 5 times
    ///   faux::when!(mock.single_arg)
    ///       .times(5)
    ///       .safe_then(|input| vec![input as i8]);
    ///
    ///   // can be called 5 times safely
    ///   for _ in 0..5 {
    ///     assert_eq!(mock.single_arg(8), vec![8]);
    ///   }
    /// }
    /// ```
    ///
    /// ## Panics
    ///
    /// ```rust should_panic
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // limit to 5 times
    ///   faux::when!(mock.single_arg)
    ///       .times(5)
    ///       .safe_then(|input| vec![input as i8]);
    ///
    ///   // panics on the 6th call
    ///   for _ in 0..6 {
    ///     assert_eq!(mock.single_arg(8), vec![8]);
    ///   }
    /// }
    /// ```
    pub fn times(mut self, times: usize) -> Self {
        self.times = MockTimes::Times(times);
        self
    }

    /// Limits mock to one call, allowing mocks to consume captured variables.
    ///
    /// Panics if the mock is called more than once.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let vec = vec![25];
    ///   //moves vec to the closure
    ///   faux::when!(mock.single_arg).once().safe_then(|_| vec);
    ///   assert_eq!(mock.single_arg(8), vec![25]);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// ```rust should_panic
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let vec = vec![25];
    ///   faux::when!(mock.single_arg).once().safe_then(|_| vec);
    ///   assert_eq!(mock.single_arg(8), vec![25]);
    ///   //panics on its 2nd call
    ///   mock.single_arg(8);
    /// }
    /// ```
    pub fn once(self) -> WhenOnce<'q, I, O> {
        WhenOnce {
            id: self.id,
            store: self.store,
            _marker: self._marker,
        }
    }
}

/// Similar to [When](struct.When) but only mocks once.
///
/// Mock closures may consume captured variables as the mock will not
/// be called more than once.
pub struct WhenOnce<'q, I, O> {
    id: &'static str,
    store: &'q mut MockStore,
    // *const for variance -- I think that's what I want.
    _marker: std::marker::PhantomData<(*const I, *const O)>,
}

impl<'q, I, O> WhenOnce<'q, I, O> {
    /// A mirror of [When.then] but the mock may consume captured
    /// variables.
    ///
    /// # Safety
    /// See [When.then's safety]
    ///
    /// [When.then]: struct.When.html#method.then
    /// [When.then's safety]: (struct.When.html#safety)
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: &u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let vec = vec![25];
    ///   //moves vec to the closure
    ///   unsafe { faux::when!(mock.single_arg).once().then(|_| vec); }
    ///   assert_eq!(mock.single_arg(&8), vec![25]);
    /// }
    /// ```
    pub unsafe fn then(self, mock: impl FnOnce(I) -> O + Send) {
        self.store.unsafe_mock_once(self.id, mock);
    }

    /// A mirror of [When.then_do] but the mock may consume captured
    /// variables.
    ///
    /// [When.then_do]: struct.When.html#method_then_do
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: &u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let vec = vec![25];
    ///   //moves vec to the closure
    ///   faux::when!(mock.single_arg).once().then_do(|| vec);
    ///   assert_eq!(mock.single_arg(&8), vec![25]);
    /// }
    /// ```
    pub fn then_do(self, mock: impl FnOnce() -> O + 'static + Send)
    where
        O: 'static,
    {
        unsafe { self.then(move |_: I| mock()) }
    }

    /// A mirror of [When.then_return] but the object does not need to
    /// be cloneable.
    ///
    /// [When.then_return]: struct.When.html#method_then_return
    ///
    /// # Usage
    ///
    /// ```rust
    /// // this does not implement Clone
    /// #[derive(PartialEq, Eq, Debug)]
    /// pub struct NonCloneableData(i32);
    ///
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: &u8) -> NonCloneableData {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   faux::when!(mock.single_arg).once().then_return(NonCloneableData(2));
    ///   assert_eq!(mock.single_arg(&8), NonCloneableData(2));
    /// }
    /// ```
    pub fn then_return(self, mock: O)
    where
        O: 'static + Send,
    {
        unsafe { self.then(move |_: I| mock) }
    }

    /// A mirror of [When.safe_then] but the mock may consume captured
    /// variables.
    ///
    /// [When.safe_then]: struct.When.html#method.safe_then
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn single_arg(&self, a: u8) -> Vec<i8> {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let vec = vec![25];
    ///   //moves vec to the closure
    ///   faux::when!(mock.single_arg).once().safe_then(|_| vec);
    ///   assert_eq!(mock.single_arg(8), vec![25]);
    /// }
    /// ```
    pub fn safe_then(self, mock: impl FnOnce(I) -> O + 'static + Send)
    where
        I: 'static,
        O: 'static,
    {
        self.store.mock_once(self.id, mock);
    }
}
