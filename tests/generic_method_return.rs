#![allow(clippy::disallowed_names)]

pub trait MyTrait {}

#[derive(Clone, PartialEq, Debug)]
struct Entity {}
impl MyTrait for Entity {}

#[derive(Clone, PartialEq, Debug)]
struct Entity2 {}
impl MyTrait for Entity2 {}

#[faux::create]
pub struct Foo {}

#[faux::create]
pub struct Bar {}

#[faux::methods]
impl Foo {
    pub fn foo<E: MyTrait>(&self, _e: E) -> E {
        todo!()
    }
    pub fn bar<E: MyTrait, F: MyTrait>(&self, _e: E, _f: F) -> Result<E, F> {
        todo!()
    }
    pub fn baz<E>(&self, _e: E) -> E
    where
        E: MyTrait,
    {
        todo!()
    }
    pub fn qux<E>(&self)
    where
        E: MyTrait,
    {
        todo!()
    }
}

#[faux::create]
struct AsyncFoo {}
#[faux::methods]
impl AsyncFoo {
    pub async fn foo<E: MyTrait>(&self, _e: E) -> E {
        todo!()
    }
    pub async fn bar<E: MyTrait, F: MyTrait>(&self, _e: E, _f: F) -> Result<E, F> {
        todo!()
    }
    pub async fn baz<E>(&self, _e: E) -> E
    where
        E: MyTrait,
    {
        todo!()
    }
    pub async fn qux<E>(&self)
    where
        E: MyTrait,
    {
        todo!()
    }

    pub async fn qux_with_arg<E>(&self, _arg: u32) -> u32
    where
        E: MyTrait,
    {
        todo!()
    }
}

#[test]
fn generics() {
    let mut foo = Foo::faux();
    faux::when!(foo.foo).then_return(Entity {});
    assert_eq!(foo.foo(Entity {}), Entity {});

    let mut bar = Foo::faux();
    faux::when!(bar.bar).then_return(Ok::<_, Entity>(Entity {}));
    assert_eq!(bar.bar(Entity {}, Entity {}), Ok(Entity {}));

    let mut baz = Foo::faux();
    faux::when!(baz.baz).then_return(Entity {});
    assert_eq!(baz.baz(Entity {}), Entity {});

    let mut qux = Foo::faux();
    faux::when!(qux.qux::<Entity>()).then(|_| {});
    qux.qux::<Entity>();
}

#[test]
fn generic_tests_async() {
    let mut foo: AsyncFoo = AsyncFoo::faux();
    faux::when!(foo.foo).then_return(Entity {});

    let mut bar = AsyncFoo::faux();
    faux::when!(bar.bar).then_return(Ok::<_, Entity>(Entity {}));

    let mut baz = AsyncFoo::faux();
    faux::when!(baz.baz).then_return(Entity {});

    let mut qux = AsyncFoo::faux();
    faux::when!(qux.qux::<Entity>()).then(|_| {});

    let mut qux_with_arg = AsyncFoo::faux();
    faux::when!(qux_with_arg.qux_with_arg::<Entity>()).then(|_| 100);
    faux::when!(qux_with_arg.qux_with_arg::<Entity>(42)).then(|_| 84);
    faux::when!(qux_with_arg.qux_with_arg::<Entity>(43)).then(|_| 86);
    futures::executor::block_on(async {
        assert_eq!(foo.foo(Entity {}).await, Entity {});
        assert_eq!(bar.bar(Entity {}, Entity {}).await, Ok(Entity {}));
        assert_eq!(baz.baz(Entity {}).await, Entity {});
        qux.qux::<Entity>().await;
        assert_eq!(qux_with_arg.qux_with_arg::<Entity>(42).await, 84);
        assert_eq!(qux_with_arg.qux_with_arg::<Entity>(43).await, 86);
        assert_eq!(qux_with_arg.qux_with_arg::<Entity>(50).await, 100);
    });
}

#[test]
fn generic_two_different_impls() {
    let mut qux_with_arg = AsyncFoo::faux();
    faux::when!(qux_with_arg.qux_with_arg::<Entity>()).then(|_| 100);
    faux::when!(qux_with_arg.qux_with_arg::<Entity2>()).then(|_| 200);
    futures::executor::block_on(async {
        assert_eq!(qux_with_arg.qux_with_arg::<Entity>(42).await, 100);
        assert_eq!(qux_with_arg.qux_with_arg::<Entity2>(42).await, 200);
    });
}

#[test]
#[should_panic(expected = "`Foo::qux<E>` was called but never stubbed")]
fn unmocked_faux_panics_with_generic_information() {
    let foo = Foo::faux();
    foo.qux::<Entity>();
}
