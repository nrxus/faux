use crate::{matcher, MockStore};

/// Similar to [WhenWithArgs](struct.WhenWithArgs) but only mocks once.
///
/// Mock closures may consume captured variables as the mock will not
/// be called more than once.
pub struct WithArgsOnce<'q, R, I, O, M> {
    id: fn(R, I) -> O,
    store: &'q mut MockStore,
    matcher: M,
}

impl<'q, R, I, O, M: matcher::AllArgs<I> + Send + 'static> WithArgsOnce<'q, R, I, O, M> {
    #[doc(hidden)]
    pub fn new(id: fn(R, I) -> O, store: &'q mut MockStore, matcher: M) -> Self {
        WithArgsOnce { id, store, matcher }
    }

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
        self.then(|| value)
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
    pub fn then(self, mock: impl FnOnce() -> O + 'static + Send)
    where
        O: 'static,
    {
        let Self {
            id, matcher, store, ..
        } = self;

        store.mock_once(id, move |input: I| {
            if let Err(message) = matcher.matches(&input) {
                panic!("{}", message)
            }

            mock()
        });
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
        self.then_unchecked(move || value)
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
    pub unsafe fn then_unchecked(self, mock: impl FnOnce() -> O + Send) {
        let Self {
            id, matcher, store, ..
        } = self;

        store.unsafe_mock_once(id, move |input: I| {
            if let Err(message) = matcher.matches(&input) {
                panic!("{}", message)
            }

            mock()
        });
    }
}
