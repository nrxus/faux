# faux &emsp; [![Latest Version]][crates.io] [![rustc 1.63+]][Rust 1.63] [![docs]][api docs] ![][build]

A library to create [mocks] out of structs.

`faux` allows you to mock the methods of structs for testing without
complicating or polluting your code.

See the [API docs] for more information.

## Getting Started

`faux` makes liberal use of unsafe Rust features, so it is only
recommended for use inside tests. To prevent `faux` from leaking into
your production code, set it as a `dev-dependency` in your
`Cargo.toml`:

```toml
[dev-dependencies]
faux = "^0.1"
```
`faux` provides two attributes:
* `#[create]`: transforms a struct into a mockable equivalent
* `#[methods]`: transforms the methods in an `impl` block into their
  mockable equivalent

Use Rust's `#[cfg_attr(...)]` to gate these attributes to the test
config only.

```rust
#[cfg_attr(test, faux::create)]
pub struct MyStructToMock { /* fields */ }

#[cfg_attr(test, faux::methods)]
impl MyStructToMock { /* methods to mock */ }
```

## Examples

```rust
mod client {
    // #[faux::create] makes a struct mockable and
    // generates an associated `faux` function
    // e.g., `UserClient::faux()` will create a mock `UserClient` instance
    #[faux::create]
    pub struct UserClient { /* data of the client */ }

    #[derive(Clone)]
    pub struct User {
        pub name: String
    }

    // #[faux::methods] makes every public method in the `impl` block mockable
    #[faux::methods]
    impl UserClient {
        pub fn fetch(&self, id: usize) -> User {
            // does some network calls that we rather not do in tests
            User { name: "".into() }
        }
    }
}

use crate::client::UserClient;

pub struct Service {
    client: UserClient,
}

#[derive(Debug, PartialEq)]
pub struct UserData {
    pub id: usize,
    pub name: String,
}

impl Service {
    fn user_data(&self) -> UserData {
        let id = 3;
        let user = self.client.fetch(id);
        UserData { id, name: user.name }
    }
}

// A sample #[test] for Service that mocks the client::UserClient
fn main() {
    // create a mock of client::UserClient using `faux`
    let mut client = client::UserClient::faux();

    // mock fetch but only if the argument is 3
    // argument matchers are optional
    faux::when!(client.fetch(3))
        // stub the return value for this mock
        .then_return(client::User { name: "my user name".into() });

    // prepare the subject for your test using the mocked client
    let subject = Service { client };

    // assert that your subject returns the expected data
    let expected = UserData { id: 3, name: String::from("my user name") };
    assert_eq!(subject.user_data(), expected);
}
```

**Due to [constraints with rustdocs], the above example tests in
`main()` rather than a `#[test]` function. In real life, the faux
attributes should be gated to `#[cfg(test)]`.**

## Features
`faux` lets you mock the return value or implementation of:

* Async methods
* Trait methods
* Generic struct methods
* Methods with pointer self types (e.g., `self: Rc<Self>`)
* Methods in external modules (but not external crates).

`faux` also provides easy-to-use argument matchers.

## Interaction with `#[derive(...)]` and auto-traits.

`faux` mocks will auto implement `Send` and `Sync` if the real
instance also implements it. Using `#[derive(...)]` for `Clone`,
`Debug`, and `Default` will also work as expected. Other derivable
traits are not supported as they are about data (e.g., `Eq`, or
`Hash`) but `faux` is about mocking behavior not data. Deriving traits
that are not part of the standard library is also not currently
supported. An escape hatch for this is to manually write the `impl`
for that trait. If you believe there is a derivable trait that `faux`
should support please file an issue explaining your use case.

`Clone` is a bit of a special case in that it does not duplicate the
stubs but instead shares them with the cloned instance. If this is not
the desired behavior for cloning mocks you may instead implement
`Clone` manually and do normal method stubbing
(`faux::when!(my_struct.clone()).then_return(/* something */)`). Note
that for the cases of exhaustable stubs (e.g.,
`faux::when!(my_struct.foo()).once()`) if either instance calls for
the stub that will count as exhausting the stub as they are shared.

## Interactions with other proc macros

While `faux` makes no guarantees that it will work with other macro
libraries, it should "just" work. There are some caveats, however. For
a quick solution, try making the `faux` attributes (e.g.
`#[faux::methods]`) the first attribute.

### Explanation

If another `proc-macro` modifies the *signature* of a method before
`faux` does its macro expansion, then it could modify the signature
into something not supported by `faux`. Unfortunately, [the order of
proc macros is not specified]. However, in practice it *seems* to
expand top-down (tested in Rust 1.42).

```rust ignore
#[faux::create]
struct Foo { /*some items here */ }

#[faux::methods]
#[another_attribute]
impl Foo {
    /* some methods here */
}
```

In the snippet above, `#[faux::methods]` will expand first followed by
`#[another_attribute]`.`faux` is effectively ignoring the other macro
and expanding based on the code you wrote.

If `#[faux::methods]` performs its expansion after another macro has
modified the `impl` block, `#[faux::methods]` receives the expanded
code. This code might contain different method signatures than what
you originally wrote. Note that the other proc macro's expansion may
create code that `faux` cannot handle (e.g. explicit lifetimes).

For a concrete example, let's look at
[`async-trait`](https://github.com/dtolnay/async-trait). `async-trait` effectively converts:

```rust ignore
async fn run(&self, arg: Arg) -> Out {
    /* stuff inside */
}
```

```rust ignore
fn run<'async>(&'async self, arg: Arg) -> Pin<Box<dyn std::future::Future<Output = Out> + Send + 'async>> {
    /* crazier stuff inside */
}
```

Because `async-trait` adds explicit lifetimes to the method signature,
which `faux` cannot handle, having `async-trait` do its expansion
first breaks `faux`. Note that even if `faux` could handle explicit
lifetimes, our signature is now so unwieldy that it would make mocks
hard to work with. Because `async-trait` just wants an `async`
function signature, and `faux` does not modify function signatures, it
is okay for `faux` to expand first.

```rust ignore
#[faux::methods]
#[async_trait]
impl MyStruct for MyTrait {
    async fn run(&self, arg: Arg) -> Out {
        /* stuff inside */
    }
}
```

If you find a proc macro that `faux` cannot handle, please open an
issue to see if `faux` is doing something unexpected that conflicts
with that macro.

## Goal

`faux` was founded on the belief that traits with single
implementations are an undue burden and an unnecessary layer of
abstraction. Thus, `faux` does not rely on trait definitions for every
mocked object, which would pollute their function signatures with
either generics or trait objects. `faux` aims to create mocks out of
user-defined structs, avoiding extra production code that exists
solely for tests.

## Inspiration

This library was inspired by [mocktopus], a mocking library for
nightly Rust that lets you mock any function. Unlike mocktopus, `faux`
works on stable Rust and deliberately only allows for mocking public
methods in structs.

[Latest Version]: https://img.shields.io/crates/v/faux.svg
[crates.io]: https://crates.io/crates/faux
[rustc 1.63+]: https://img.shields.io/badge/rustc-1.63+-blue.svg
[Rust 1.63]: https://blog.rust-lang.org/2022/08/11/Rust-1.63.0.html
[Latest Version]: https://img.shields.io/crates/v/faux.svg
[docs]: https://img.shields.io/badge/api-docs-blue.svg
[api docs]: https://docs.rs/faux/
[mocktopus]: https://github.com/CodeSandwich/Mocktopus
[build]: https://github.com/nrxus/faux/workflows/test/badge.svg
[constraints with rustdocs]: https://github.com/rust-lang/rust/issues/45599
[the order of proc macros is not specified]: https://github.com/rust-lang/reference/issues/578
[mocks]: https://martinfowler.com/articles/mocksArentStubs.html
