# faux &emsp; [![Latest Version]][crates.io] [![rustc 1.45+]][Rust 1.45] [![docs]][api docs] ![][build]

faux is a traitless mocking library for stable Rust. It was inspired
by [mocktopus], a mocking library for nightly Rust that lets you mock
any function. Unlike mocktopus, faux deliberately only allows for
mocking public methods in structs.

See the [API docs] for more information.

**faux is in its early alpha stages, so there are no guarantees of API
stability.**

## Setup

faux will modify existing code at compile time to transform structs
and their methods into mockable versions of themselves. faux makes
liberal use of unsafe Rust features, so it is only recommended for use
inside of tests. Add `faux` as a dev-dependency in `Cargo.toml`to
prevent usage in production code:

``` toml
[dev-dependencies]
faux = "0.0.6"
```

faux provides two attributes: `create` and `methods`. Use these
attributes for tagging your struct and its impl block
respectively. Use Rust's `#[cfg_attr(...)]` to gate these attributes
to the test config only.

``` rust
#[cfg_attr(test, faux::create)]
pub struct MyStructToMock { /* fields */ }

#[cfg_attr(test, faux::methods)]
impl MyStructToMock { /* methods to mock */ }
```


## Usage

```rust
mod client {
    // creates a mockable version of `UserClient`
    // generates an associated function, `UserClient::faux`, to create a mocked instance
    #[faux::create]
    pub struct UserClient { /* data of the client */ }

    pub struct User {
        pub name: String
    }

    // creates mockable version of every method in the impl
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

    // set up what the mock should return
    faux::when!(client.fetch).safe_then(|id| {
        assert_eq!(id, 3, "expected UserClient.fetch to receive user #3");
        client::User { name: "my user name".into() }
    });

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

## Interactions With Other Proc Macros

`faux` makes no guarantees that it will work with other macro
libraries. `faux` in theory should "just" work although with some
caveats, in particular if they modify the *signature* of
methods.

Unfortunately, [the order of proc macros is not
specified]. However, in practive it *seems* to expand top-down (tested
in Rust 1.42).

```rust ignore
#[faux::create]
struct Foo { /*some items here */ }

#[faux::methods]
#[another_attribute]
impl Foo {
    /* some methods here */
}

# fn main() {}
```

In the snippet above, `#[faux::methods]` will expand first followed by
`#[another_attribute]`.

If `faux` does its expansion first then `faux` will effectively ignore
the other macro and expand based on the code that the user wrote. If
you want `faux` to treat the code in the `impl` block (or the
`struct`) as-is, before the expansion then put it on the top.

If `faux` does its expansion after, then `faux` will morph the
expanded version of the code, which might have a different signature
than what you originally wrote. Note that the other proc macro's
expansion may create code that `faux` cannot handle (e.g., explicit
lifetimes).

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

Because `async-trait` modifies the signature of the function to a
signature that `faux` cannot handle (explicit lifetimes) then having
`async-trait` do its expansion *before* `faux` would make `faux` not
work. Note that even if `faux` could handle explicit lifetimes, our
signature now it's so unwieldy that it would make mocks hard to work
with. Because `async-trait` just wants an `async` function signature,
and `faux` does not modify function signatures, it is okay for `faux`
to expand first.

```rust ignore
#[faux::methods]
#[async_trait]
impl MyStruct for MyTrait {
    async fn run(&self, arg: Arg) -> Out {
        /* stuff inside */
    }
}
```

Since no expansions came before, `faux` sees an `async` function,
which it supports. `faux` does its magic assuming this is a normal
`async` function, and then `async-trait` does its magic to convert the
signature to something that can work on trait `impl`s.

If you find a procedural macro that `faux` cannot handle please submit
an issue to see if `faux` is doing something unexpected that conflicts
with that macro.

## Goal

faux was founded on the belief that traits with single implementations
are an undue burden and an unnecessary layer of abstraction. It aims
to create mocks out of user-defined structs, avoiding extra production
code that exists solely for tests. In particular, faux does not rely
on trait definitions for every mocked object, which would pollute
their function signatures with either generics or trait objects.

[Latest Version]: https://img.shields.io/crates/v/faux.svg
[crates.io]: https://crates.io/crates/faux
[rustc 1.45+]: https://img.shields.io/badge/rustc-1.45+-blue.svg
[Rust 1.45]: https://blog.rust-lang.org/2020/07/16/Rust-1.45.0.html
[Latest Version]: https://img.shields.io/crates/v/faux.svg
[docs]: https://img.shields.io/badge/api-docs-blue.svg
[api docs]: https://docs.rs/faux/
[mocktopus]: https://github.com/CodeSandwich/Mocktopus
[build]: https://github.com/nrxus/faux/workflows/test/badge.svg
[constraints with rustdocs]: https://github.com/rust-lang/rust/issues/45599
[the order of proc macros is not specified]: https://github.com/rust-lang/reference/issues/578
