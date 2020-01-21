# 🎉 🎉 faux release 🎉 🎉

## What is faux?

[`faux`] is a Rust mocking framework for creating mock objects out of
user-defined structs without bloating production code.

[`faux`]: https://github.com/nrxus/faux

## Why mock objects?

Mock objects are fake versions of objects created for testing. For
example, if your code uses structs that access your file system, make
network requests, or render graphics, it is often useful to mock the
methods of those structs. Mocking can help make your tests into true
"units". Unit tests should run quickly and produce the same result
every time without relying on external dependencies. For a deeper dive
into mocks, read this [post].

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
    //creates a mocked NetworkClient
    let mut client = NetworkClient::faux();

    //mock the method in client
    faux::when!(client.fetch_id_matching).safe_then(|i| {
        assert_eq!(i, 3, "expected service to send '3'");
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
#   //creates a mocked NetworkClient
#    let mut client = NetworkClient::faux();
#
#    //mock the method in client
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

In the above code snippet, we have succesfully mocked `NetworkClient`
to call a closure specified in the test code. This avoids making the
request that `NetworkClient::fetch_id_matching` would normally make,
thus making our tests dependable and focused. Our test relies does not
rely external dependency of a network call.

`faux` provides users with two attributes, `#[faux::create]` and
`#[faux::methods]`. `#[faux::create]` is required on any struct that
needs to be mocked. `#[faux::methods]` is required on an `impl` block
whose public methods need to be mocked. `faux` also provides a `when!`
macro to mock the methods that were made mockable by
`#[faux::methods]`. More info in the [docs].

## How is faux different than ${existing mocking framework}?

**DISCLAIMER: this section is based on the author's knowledge of Rust
mocking frameworks as of January 2020. Apologies in advance for any
frameworks I may have overlooked in my searches.**

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
does the right thing. However, we do not want to do the fetching work
in `fetch_id_matching` on every test, since making a network request
in our test would be both slow and unreliable. This means we need two
different implementations of `NetworkClient`, one for tests and one
for production. Using traits, we could write the following,

```rust
trait INetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32;
}

struct NetworkClient {
    /* data here */
}

impl INetworkClient for NetworkClient {
    fn fetch_id_matching(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}

struct Service<C: INetworkClient> {
    client: C,
}

impl<C: INetworkClient> Service<C> {
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
impl INetworkClient for MockNetworkClient {
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
# impl INetworkClient for MockNetworkClient {
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

Unfortunately, we have now changed our *production* code to accomodate
our *tests*, not because this is a better design but because of
testing requirements. Tests should *guide* the design of your code,
but should not force you to add undue complexity that only tests
benefit from. Now every user of `Service` needs to explicitly call out
the `INetworkClient`, thus cluttering the function/struct signature of
anything dealing with `Service`. In your production code, there is
always only a single implementation of `INetworkClient`, making the
trait an unncessary layer of abstraction.

While the code above is a simple example, imagine having to add mock
interfaces to all the structs in a mature codebase. Most mocking
frameworks for rust currently are based on this approach, and aim at
automatically generating the mock structs from traits, but you still
need to provide hand-written traits for every struct that needs to be
mocked, and you still have to deal with generics and traits in your
function/struct signatures.

`faux` takes a different approach. It provides a separate implementation
of your struct by transforming your struct into one that can be
mocked. This transformation can (and should be!) gated to only the
`test` cfg, thus having zero impact to your production code.

## Closing note

`faux` is in its very early stages, still exploring what all is
possible. While an attempt will be done to keep API stability between
releases, no promises will be made in case a different API turns out
to be a better fit.

The [docs] are the source of true for what is capable.

See the [issues] in Github for an updated list of limitations and get
an idea of what might be coming next.

`faux` is always open to feedback, so feel free to open issues/PRs
liberally.

A huge thanks to the author of [mocktopus], another mocking framework
for Rust that also does not use traits as its building block for
mocking. `faux` was heavily inspired by seeing what mocktopus was
capable of.

[mocktopus]: https://github.com/CodeSandwich/Mocktopus
[docs]: https://docs.rs/faux/
[issues]: https://github.com/nrxus/faux/issues
