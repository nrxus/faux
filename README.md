# faux &emsp; [![Latest Version]][crates.io] [![rustc 1.40+]][Rust 1.40] [![docs]][api docs] ![][build]

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
faux = "0.0.4"
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

## Goal

faux was founded on the belief that traits with single implementations
are an undue burden and an unnecessary layer of abstraction. It aims
to create mocks out of user-defined structs, avoiding extra production
code that exists solely for tests. In particular, faux does not rely
on trait definitions for every mocked object, which would pollute
their function signatures with either generics or trait objects.

[Latest Version]: https://img.shields.io/crates/v/faux.svg
[crates.io]: https://crates.io/crates/faux
[rustc 1.40+]: https://img.shields.io/badge/rustc-1.40+-blue.svg
[Rust 1.40]: https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html
[Latest Version]: https://img.shields.io/crates/v/faux.svg
[docs]: https://img.shields.io/badge/api-docs-blue.svg
[api docs]: https://docs.rs/faux/
[mocktopus]: https://github.com/CodeSandwich/Mocktopus
[build]: https://github.com/nrxus/faux/workflows/test/badge.svg
[constraints with rustdocs]: https://github.com/rust-lang/rust/issues/45599
