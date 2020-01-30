# 🎉 🎉 Introducing faux 🎉 🎉

## What is faux?

[`faux`] is a traitless Rust mocking framework for creating mock
objects out of user-defined structs.

[`faux`]: https://github.com/nrxus/faux

## Why mock?

Mock objects are test versions of objects that contain fake
implementations of their behavior. For example, if your code accesses
your file system, makes network requests, or performs other expensive
actions, it is often useful to mock that behavior. Mocking can help
make your tests into true unit tests that run quickly and produce the
same result every time without relying on external dependencies. For a
deeper dive into mocks, read this [post].

[post]: https://martinfowler.com/articles/mocksArentStubs.html

## Example

```rust
# extern crate faux;
#
#[cfg_attr(test, faux::create)]
# #[faux::create]
pub struct NetworkClient {
    /* data here */
}

#[cfg_attr(test, faux::methods)]
# #[faux::methods]
impl NetworkClient {
    pub fn fetch_id_matching(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}

struct Service {
    client: NetworkClient,
}

impl Service {
    fn do_stuff(&self) -> i32 {
        self.client.fetch_id_matching(3)
    }
}

#[cfg(test)]
#[test]
fn service_does_the_right_thing() {
    // creates a mocked NetworkClient
    let mut client = NetworkClient::faux();

    faux::when!(client.fetch_id_matching).safe_then(|i| {
        // we want to test do_stuff(), which should always call
        // fetch_id_matching with the input 3.
        assert_eq!(i, 3, "expected service to send '3'");
        // mock fetch_id_matching to always return 10
        10
    });

    // create your service using the mocked client
    // the service is the subject under test
    let subject = Service { client };
    let id = subject.do_stuff();
    assert_eq!(id, 10);
}
#
# fn main() {
#    // creates a mocked NetworkClient
#    let mut client = NetworkClient::faux();
#
#    // mock fetch_id_matching
#    faux::when!(client.fetch_id_matching).safe_then(|i| {
#         assert_eq!(i, 3, "expected service to send '3'");
#         10
#    });
#
#    // create your service using the mocked client
#    // the service is the subject under test
#    let subject = Service { client };
#    let id = subject.do_stuff();
#    assert_eq!(id, 10);
# }
```

By adding `faux` attributes, we have succesfully mocked
`NetworkClient::fetch_id_matching` to instead call a closure specified
in the test code and always return 10. Unlike the real method, the
mock does not make a network request. Thus, our test remains
dependable, focused, and free from external dependencies.

`faux` provides users with two attributes: `#[faux::create]` and
`#[faux::methods]`. `#[faux::create]` is required on any struct that
needs to be mocked and `#[faux::methods]` is required on its `impl`
block. `faux` also provides a `when!` macro to mock methods that
were made mockable by `#[faux::methods]`. See the [docs] for more information.

## How is faux different than ${existing mocking framework}?

**DISCLAIMER: this section is based on the author's knowledge of Rust
mocking frameworks as of January 2020. Apologies in advance for any
frameworks that were overlooked.**

Currently in Rust, mocking depends heavily on traits.

```rust
struct NetworkClient {
    /* data here */
}

impl NetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}

struct Service {
    client: NetworkClient,
}

impl Service {
    fn do_stuff(&self) -> i32 {
        self.client.fetch_id_matching(3)
    }
}
```

In the code snippet above, we want to test `Service` to make sure it
does the right thing. However, we want to avoid the expensive work
done in `fetch_id_matching`, since making a network call in our test
would be both slow and unreliable. This means we need two different
implementations of `NetworkClient`: one for tests, and one for
production. Using a common trait for the two implementations, we could
write the following:

```rust
trait NetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32;
}

struct NetworkClient {
    /* data here */
}

impl NetworkClient for NetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}

struct Service<C: NetworkClient> {
    client: C,
}

impl<C: NetworkClient> Service<C> {
    fn do_stuff(&self) -> i32 {
        self.client.fetch_id_matching(3)
    }
}

#[cfg(test)]
struct MockNetworkClient {
    mocked_fetch_id_matching_result: i32,
    mocked_fetch_id_matching_argument: std::cell::Cell<u32>,
}

#[cfg(test)]
impl NetworkClient for MockNetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32 {
        self.mocked_fetch_id_matching_argument.set(a);
        self.mocked_fetch_id_matching_result
    }
}

# struct MockNetworkClient {
#     mocked_fetch_id_matching_result: i32,
#     mocked_fetch_id_matching_argument: std::cell::Cell<u32>,
# }
#
# impl NetworkClient for MockNetworkClient {
#     fn fetch_id_matching(&self, a: u32) -> i32 {
#         self.mocked_fetch_id_matching_argument.set(a);
#         self.mocked_fetch_id_matching_result
#     }
# }
#

#[cfg(test)]
#[test]
fn service_does_the_right_thing() {
    //creates a mocked NetworkClient
    let client = MockNetworkClient {
        mocked_fetch_id_matching_argument: std::cell::Cell::default(),
        mocked_fetch_id_matching_result: 10,
    };

    // create your service using the mocked client
    // the service is the subject under test
    let subject = Service { client };
    let id = subject.do_stuff();
    assert_eq!(id, 10);
}
#
# fn main() {
#     //creates a mocked NetworkClient
#     let client = MockNetworkClient {
#         mocked_fetch_id_matching_argument: std::cell::Cell::default(),
#         mocked_fetch_id_matching_result: 10,
#     };
#
#     // create your service using the mocked client
#     // the service is the subject under test
#     let subject = Service { client };
#     let id = subject.do_stuff();
#     assert_eq!(id, 10);
# }
```

Unfortunately, we have now changed our *production* code to
accommodate our *tests*, not because this is a better design but
because of testing requirements. Tests should *guide* the design of
your code without forcing undue complexity that only benefits the
tests. Now, every user of `Service` needs to explicitly call out the
`NetworkClient` trait, thus cluttering the function/struct signature
of anything dealing with `Service`. Furthermore, the `NetworkClient`
trait is an unnecessary layer of abstraction for your production code,
which only uses one implementation of the trait.

While the code above is a simple example, imagine having to add mock
interfaces to all the structs in a mature codebase. Most mocking
frameworks for Rust are currently based on this approach. Although
most can automatically generate the mock structs from traits, you
still need to define hand-written traits for every mockable struct,
and you still have to deal with generics and traits in your
function/struct signatures.

`faux` takes a different approach by transforming your struct and its
methods into mockable versions of themselves. These transformations
can be (and should be!) gated to only the `test` cfg, thus having zero
impact on your production code.

## Closing note

`faux` is in a very early stage of development, and definitely does
not cover all the possibilities of a traitless mocking
framework. Thus, there is no guarantee of API stability between
releases, although every attempt will be made to keep the API
consistent. Please read the [docs] for the most up to date information
on `faux` functionality.

See the [issues] in Github for an updated list of limitations and to get
an idea of what might be coming next.

Feedback is always welcome, so feel free to open issues or send PRs.

A huge thanks to [mocktopus], another traitless Rust mocking
framework, which was a huge inspiration behind the creation of `faux`.

[mocktopus]: https://github.com/CodeSandwich/Mocktopus
[docs]: https://docs.rs/faux/
[issues]: https://github.com/nrxus/faux/issues
