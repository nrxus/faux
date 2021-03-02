use crate::{mock::MockTimes, mock_store::MockStore};

/// Provides methods to override the return or implementation of the
/// mocked method.
///
/// Created using [when!].
///
/// By default, all methods are mocked indefinitely and the mock
/// closures may not consume captured variables. See the [times] and
/// [once] methods to override these default.
///
/// [when!]: macro.when.html
/// [once]: #method.once
/// [times]: #method.times
pub struct When<'q, I, O> {
    id: &'static str,
    store: &'q mut MockStore,
    // contravariant with I but covariant with O
    _marker: std::marker::PhantomData<fn(I) -> O>,
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

    /// Sets the return value for the mocked method.
    ///
    /// Requires the value to be static. For a more lax but unsafe
    /// alternative, see: [then_unchecked_return]
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
    /// ```
    ///
    /// [then_unchecked_return]: #methods.then_unchecked_return
    ///
    pub fn then_return(self, value: O)
    where
        O: 'static + Send + Clone,
    {
        unsafe { self.then_unchecked_return(value) }
    }

    /// Sets the closure to be called when the mocked method is
    /// invoked.
    ///
    /// The input for the closure is a tuple of all its non-receiver
    /// parameters
    ///
    /// The provided mock must be static and it must be mocking a
    /// method with static output. For a more lax but unsafe
    /// alternative, see: [then_unchecked].
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
    ///
    ///     pub fn out_ref(&self) -> &u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   // methods with no params
    ///   faux::when!(mock.no_args).then(|_| 5);
    ///   assert_eq!(mock.no_args(), 5);
    ///
    ///   // methods with a single param
    ///   faux::when!(mock.single_arg).then(|input| vec![input as i8]);
    ///   assert_eq!(mock.single_arg(8), vec![8]);
    ///
    ///   // methods with multiple params - some can be references
    ///   faux::when!(mock.multi_args).then(|(&a, _)| a as u32);
    ///   assert_eq!(mock.multi_args(&5, 2), 5);
    ///
    ///   // cannot mock methods that return references
    ///   // let x = 5;
    ///   // faux::when!(mock.out_ref).then(|_| &x);
    /// }
    /// ```
    ///
    /// [then_unchecked]: #methods.then_unchecked
    ///
    pub fn then(self, mock: impl FnMut(I) -> O + 'static + Send)
    where
        O: 'static,
    {
        self.store.mock(self.id, mock, self.times);
    }

    /// Analog of [then_return] that allows returning non-static
    /// outputs.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn multi_args(&mut self, a: &i32, b: i8) -> &u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let x = 5;
    ///   unsafe { faux::when!(mock.multi_args).then_unchecked_return(&x) }
    ///   assert_eq!(*mock.multi_args(&2, 3), x);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The lifetime of the returned object is not checked and can
    /// cause memory safety issues if used incorrectly.
    ///
    /// If the owner of the borrowed data is dropped while the
    /// captured reference is still accessible, a use-after-free
    /// violation will be triggered.
    ///
    /// This method can also cause aliasing issues where multiple
    /// mutable references are held for the same object.
    ///
    /// ### Example:
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn out_ref(&self) -> &u32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let x = 5;
    ///   unsafe { faux::when!(mock.out_ref).then_unchecked_return(&x) }
    ///   std::mem::drop(x);
    ///   // assert_eq!(*mock.ref_return(), 5); // <~~ UB: use after free
    /// }
    /// ```
    ///
    /// [then_return]: #method.then_return
    ///
    pub unsafe fn then_unchecked_return(self, value: O)
    where
        O: Send + Clone,
    {
        self.then_unchecked(move |_: I| value.clone())
    }

    /// Analog of [then] that allows using non-static closures or
    /// mocking methods with non-static outputs
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     pub fn out_ref(&mut self) -> &i32 {
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
    ///   unsafe { faux::when!(mock.out_ref).then_unchecked(|_| &x) }
    ///   assert_eq!(*mock.out_ref(), x);
    /// }
    ///
    /// ```
    ///
    ///
    /// # Safety
    ///
    /// The lifetimes of the outputs and captured variables are not
    /// checked. While this gives the caller maximum flexibility when
    /// mocking, it is *not* memory safe when used incorrectly.
    ///
    /// If the mocked method is called after its captured variables
    /// are dropped then a use-after-fress violation will be
    /// triggered.
    ///
    /// Relationships between inputs, outputs, and captured variable
    /// lifetimes are lost. This allows for easy violations of Rust's
    /// aliasing checks, creating undefined behavior.
    ///
    /// ### Example:
    ///
    /// ```rust
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
    ///   unsafe { faux::when!(mock.out_ref).then_unchecked(|i| i) }
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
    ///
    /// [then]: #methods.then
    ///
    pub unsafe fn then_unchecked(self, mock: impl FnMut(I) -> O + Send) {
        self.store.unsafe_mock(self.id, mock, self.times);
    }

    /// Limits the number of times a mock is active.
    ///
    /// Calls past the limit result in a panic.
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
    ///       .then(|input| vec![input as i8]);
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
    /// Panics if the mock is called more times than the specified
    /// number of times
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
    ///       .then(|input| vec![input as i8]);
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
    ///   faux::when!(mock.single_arg).once().then(|_| vec);
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
    ///   faux::when!(mock.single_arg).once().then(|_| vec);
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
    _marker: std::marker::PhantomData<fn(I) -> O>,
}

impl<'q, I, O> WhenOnce<'q, I, O> {
    /// Analog of [When.then_return] where the value does not need to
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
    pub fn then_return(self, value: O)
    where
        O: 'static + Send,
    {
        unsafe { self.then_unchecked_return(value) }
    }

    /// Analog of [When.then] where the mock may consume captured
    /// variables.
    ///
    /// [When.then]: struct.When.html#method.then
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
    ///   faux::when!(mock.single_arg).once().then(move |_| vec);
    ///   assert_eq!(mock.single_arg(8), vec![25]);
    /// }
    /// ```
    pub fn then(self, mock: impl FnOnce(I) -> O + 'static + Send)
    where
        O: 'static,
    {
        self.store.mock_once(self.id, mock);
    }

    /// Analog of [When.then_unchecked_return] where the value does
    /// not need to be cloneable.
    ///
    /// # Usage
    ///
    /// ```rust
    /// #[faux::create]
    /// pub struct Foo {}
    ///
    /// #[faux::methods]
    /// impl Foo {
    ///     // &mut T never implements Clone
    ///     pub fn single_arg(&self, a: &u8) -> &mut i32 {
    ///       /* implementation code */
    ///       # panic!()
    ///     }
    /// }
    ///
    /// fn main() {
    ///   let mut mock = Foo::faux();
    ///
    ///   let mut x = 50;
    ///   unsafe { faux::when!(mock.single_arg).once().then_unchecked_return(&mut x) }
    ///   assert_eq!(*mock.single_arg(&8), 50);
    /// }
    /// ```
    ///
    /// # Safety
    /// See [When.then_unchecked_return's safety]
    ///
    /// [When.then_unchecked_return]: struct.When.html#method_then_unchecked_return
    /// [When.then_unchecked_return's safety]: struct.When.html#safety
    ///
    pub unsafe fn then_unchecked_return(self, value: O)
    where
        O: Send,
    {
        self.then_unchecked(move |_: I| value)
    }

    /// Analog of [When.then_unchecked] where the mock may consume
    /// captured variables.
    ///
    /// [When.then_unchecked]: struct.When.html#method.then
    /// [When.then_unchecked's safety]: struct.When.html#safety-1
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
    ///   unsafe { faux::when!(mock.single_arg).once().then_unchecked(|_| vec); }
    ///   assert_eq!(mock.single_arg(&8), vec![25]);
    /// }
    /// ```
    ///
    /// # Safety
    /// See [When.then_unchecked's safety]
    ///
    pub unsafe fn then_unchecked(self, mock: impl FnOnce(I) -> O + Send) {
        self.store.unsafe_mock_once(self.id, mock);
    }
}
