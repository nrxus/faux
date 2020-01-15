# faux &emsp; [![Latest Version]][crates.io] [![faux: rustc 1.40+]][Rust 1.40] [![docs]][docs.rs] ![][build]

[Latest Version]: https://img.shields.io/crates/v/faux.svg
[crates.io]: https://crates.io/crates/faux
[faux: rustc 1.40+]: https://img.shields.io/badge/faux-rustc_1.40+-blue.svg
[Rust 1.40]: https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html
[Latest Version]: https://img.shields.io/crates/v/faux.svg
[docs]: https://img.shields.io/badge/api-docs-blue.svg
[docs.rs]: https://docs.rs/faux/
[build]: https://github.com/nrxus/faux/workflows/test/badge.svg

A struct mocking library for stable Rust.

This library was inspired by [mocktopus], a mocking library for
nightly rust that lets you mock any function. Unlike mocktopus, faux
deliberately only allows for mocking public methods in structs rather
than any function.

**Faux is in its early alpha stages and there are no
guarantees of API stability**

```rust
mod client {
    // #[cfg_attr(test, faux::create)] for mocking on tests only
    #[faux::create]
    pub struct UserClient { /* data of the client */ }

    pub struct User {
        pub name: String
    }

    // #[cfg_attr(test, faux::create)] for mocking on tests only
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
    // mutable to mutate the mocks inside it
    let mut client = client::UserClient::faux();

    faux::when!(client.fetch).safe_then(|id| {
        assert_eq!(id, 3, "expected UserClient.fetch to receive user #3");
        client::User { name: "my user name".into() }
    });

    let service = Service { client };
    let data = service.user_data();
    assert_eq!(data.id, 3);
    assert_eq!(data.name, String::from("my user name"));
}
```

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

[mocktopus]: https://github.com/CodeSandwich/Mocktopus
