use crate::{
    matcher::InvocationMatcher,
    mock::{self, stub, Stub},
};

/// Similar to [When](struct.When), but only stubs once.
///
/// Stubbed values do not need to be cloneable. Stubbed
/// implementations may consume variables.
///
/// Do *NOT* rely on the signature of `Once`. While changing the
/// methods of `Once` will be considered a breaking change, changing
/// the generics within `Once` will not.
pub struct Once<'m, R, I, O, M: InvocationMatcher<I>> {
    id: fn(R, I) -> O,
    name: &'static str,
    store: &'m mut mock::Store<'static>,
    matcher: M,
}

impl<'m, R, I, O, M: InvocationMatcher<I> + Send + 'static> Once<'m, R, I, O, M> {
    #[doc(hidden)]
    pub fn new(
        id: fn(R, I) -> O,
        name: &'static str,
        store: &'m mut mock::Store<'static>,
        matcher: M,
    ) -> Self {
        Once {
            id,
            name,
            store,
            matcher,
        }
    }

    /// Analog of [When.then_return] where the value does not need to
    /// be cloneable.
    ///
    /// [When.then_return]: struct.When.html#method_then_return
    ///
    /// # Examples
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

    /// Analog of [When.then] where the stub may consume captured
    /// variables.
    ///
    /// [When.then]: struct.When.html#method.then
    ///
    /// # Examples
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
    pub fn then(self, stub: impl FnOnce(I) -> O + 'static + Send)
    where
        O: 'static,
    {
        self.add_stub(Box::new(stub))
    }

    /// Analog of [When.then_unchecked_return] where the value does
    /// not need to be cloneable.
    ///
    /// # Examples
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
    /// See [When.then_unchecked_return's safety].
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

    /// Analog of [When.then_unchecked] where the stub may consume
    /// captured variables.
    ///
    /// [When.then_unchecked]: struct.When.html#method.then
    /// [When.then_unchecked's safety]: struct.When.html#safety-1
    ///
    /// # Examples
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
    /// See [When.then_unchecked's safety].
    ///
    pub unsafe fn then_unchecked(self, stub: impl FnOnce(I) -> O + Send) {
        let stub: Box<dyn FnOnce(I) -> O + Send> = Box::new(stub);
        // pretend the lifetime is 'static
        self.add_stub(std::mem::transmute(stub));
    }

    fn add_stub(self, stub: Box<dyn FnOnce(I) -> O + Send + 'static>) {
        self.store
            .get_or_create(self.id, self.name)
            .add_stub(Stub::new(stub::Answer::Once(stub), self.matcher));
    }
}
