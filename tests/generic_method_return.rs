#![allow(clippy::disallowed_names)]

pub trait MyTrait {}

#[derive(Clone, PartialEq, Debug)]
struct Entity {}
impl MyTrait for Entity {}

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
    pub fn baz<E>(&self, _e: E) -> E where E: MyTrait {
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
    pub async fn baz<E>(&self, _e: E) -> E where E: MyTrait {
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
}

#[test]
fn generic_tests_async() {
    let mut foo: AsyncFoo = AsyncFoo::faux();
    faux::when!(foo.foo).then_return(Entity {});

    let mut bar = AsyncFoo::faux();
    faux::when!(bar.bar).then_return(Ok::<_, Entity>(Entity {}));

    let mut baz = AsyncFoo::faux();
    faux::when!(baz.baz).then_return(Entity {});
    futures::executor::block_on(async {
        assert_eq!(foo.foo(Entity {}).await, Entity {});
        assert_eq!(bar.bar(Entity {}, Entity {}).await, Ok(Entity {}));
        assert_eq!(baz.baz(Entity {}).await, Entity {});
    });
}
