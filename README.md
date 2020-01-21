# faux &emsp; [![Latest Version]][crates.io] [![rustc 1.40+]][Rust 1.40] [![docs]][api docs] ![][build]

A struct mocking library for stable Rust.

Faux was inspired by [mocktopus], a mocking library for nightly rust
that lets you mock any function. Unlike mocktopus, faux deliberately
only allows for mocking public methods in structs rather than any
function.

See the [api docs] for more information.

**Faux is in its early alpha stages and there are no guarantees of API
stability**

## Setup

Faux will modify your existing code at compile time to transform your
struct and its methods into mockable versions of themselves. Faux
makes liberal use of unsafe rust features and it is only recommended
for use inside of tests. Add `faux` as a dev-dependency under cargo to
prevent any uses of it in your production code.

In your `Cargo.toml`:

``` toml
[dev-dependencies]
faux = "0.0.2"
```

Faux provides two attributes: `create` and `methods`. Use these
attributes for tagging your struct and its impl block
respectively. Use rust's `#[cfg_attr(...)]` to gate these attributes
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

**The above example does not gate the mock creation to only the test
config due to [constraints with rustdocs]. This is also why the above
example tests in `main()` rather than a `#[test]` function. In real
life, these attributes should be gated to test only.**

## Goal

Faux aims at providing the user with the power to create mocks out of
their structs for testing without the need to change their production
code for testing-purposes only. In particular, faux avoids forcing the
user to create traits to define every type they want mocked, and then
pollute their function signatures with either generics or trait
object.

It is the belief of the author that if a trait is only ever
implemented by a single object, then that trait is an undue
burden. Having to change your function/struct signatures to support
generics in production code when only tests would ever use a different
type should be an anti-pattern.

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
