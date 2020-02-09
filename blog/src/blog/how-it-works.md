# faux - an inside look

## What is faux?

[`faux`] is a traitless Rust mocking framework for creating mock
objects out of user-defined structs. For more on faux's capabilities,
take a look at the [release blog post] or the [documentation].

`faux` creates mocks of your structs to be used in unit tests, making
them fast and reliable.

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
    pub fn fetch(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}

struct Service {
    client: NetworkClient,
}

impl Service {
    fn do_stuff(&self) -> i32 {
        self.client.fetch(3)
    }
}

#[cfg(test)]
#[test]
fn service_does_the_right_thing() {
    let mut client = NetworkClient::faux();

    faux::when!(client.fetch).safe_then(|i| {
        assert_eq!(i, 3, "expected service to send '3'");
        10
    });

    let subject = Service { client };
    let id = subject.do_stuff();
    assert_eq!(id, 10);
}
#
# fn main() {
#    let mut client = NetworkClient::faux();
#
#    faux::when!(client.fetch).safe_then(|i| {
#         assert_eq!(i, 3, "expected service to send '3'");
#         10
#    });
#
#    let subject = Service { client };
#    let id = subject.do_stuff();
#    assert_eq!(id, 10);
# }
```

## How does it work?

***DISCLAIMER:** this is a simplified version of how `faux` works as
of February, 2020, which may change in future versions. To see the
most up to date transformations of your code, use `cargo-expand`*

`faux` uses attributes to transform your structs into mockable
versions of themselves at compile time.

The rest of this section focuses on code that looks like this:

```rust
pub struct NetworkClient {
    /* data here */
}

impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient {
            /* data here */
        }
    }

    pub fn fetch(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}
```

`faux`, or any other mocking framework, needs to do two things to the
code snippet above: create a fake version of `NetworkClient`, and
provide a way to inject fake implementations of its methods.

### Create mockable structs

`faux` provides the attribute macro `#[faux::create]` to transform a
struct definition into a mockable version of itself.

```rust
# extern crate faux;
#[faux::create]
pub struct NetworkClient {
    /* data here */
}
```

From `faux`'s perspective, a mockable version of a struct:

* Is indistinguishable from the original struct, from a user's perspective
* Can instantiate the original version; we do not always want a mocked instance
* Can instantiate a mocked version without any additional data

```rust
// same name so no one can tell the difference
pub struct NetworkClient(MaybeNetworkClient);

enum MaybeNetworkClient {
    // a fake does not need any data about the real network client
    Fake,
    // in case the user wants a real network client
    Real(RealNetworkClient)
}

// save the real definition somewhere else so it may still be created
struct RealNetworkClient {
    /* data here */
}

impl NetworkClient {
    // provide a method to create a fake instance of NetworkClient
    fn fake_please() -> NetworkClient {
        NetworkClient(MaybeNetworkClient::Fake)
    }
}
```

The code snippet above shows an implementation that satisfies the
mockable requirements for `NetworkClient`.

* Indistinguishable from the original struct
  * As long as no one tries to access any public fields
  * External information must be kept the same (i.e., visibility,
    attributes)
* Real instances can be created
  * The internal enum can be either a fake or a real instance
  * The real definition is kept in a struct with a different name for
    instantiation
* Mock instances can be created
  * The fake variant of the internal enum knows nothing about
    `RealNetworkClient`

The snippet above was a simplified version of what `#[faux::create]`
would do to `NetworkClient`.

1. Check that all the fields are private; fail to compile otherwise
2. Clones the definition of the struct
3. Rename the original definition such that it is saved elsewhere
4. Replace the cloned definition's fields with an enum of a fake or
   real instance.

### Create mockable methods

`faux` provides the attribute macro `#[faux::methods]` to transform
method definitions inside an impl block into mockable versions of
themselves.

```rust
# extern crate faux;
# #[faux::create]
# pub struct NetworkClient {}
#[faux::methods]
impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient {
            /* data here */
        }
    }

    pub fn fetch(&self, a: u32) -> i32 {
        /* does some complicated stuff, maybe network calls */
        # 5
    }
}
#
# fn main() {}
```

From `faux`'s perspective a mockable version of a mock:

* Is indistinguishable from the original method, from a user's
  perspective
* Can call the real method; we do not always want a mocked method
* Can run arbitrary code provided by the user

Following the hand written mockable struct from the previous section,
to get a mockable method we could hand write this:

```rust
# pub struct NetworkClient(MaybeNetworkClient);
# enum MaybeNetworkClient {
#    Fake,
#    Real(RealNetworkClient)
# }
# pub struct RealNetworkClient {}
// the numbers in the comments represent sections
// that will be explained in further detail later
impl NetworkClient {
    // (1)
    pub fn new() -> Self {
        Self(MaybeNetworkClient::Real(RealNetworkClient::new()))
    }

    // (2)
    pub fn fetch(&self, a: u32) -> i32 {
        // proxy to the real method for real instances
        // somehow get fake data when it is a mocked instance
        match self {
            Self(MaybeNetworkClient::Real(real)) => real.fetch(a),
            Self(MaybeNetworkClient::Fake) => {
                /* somehow get the fake data */
                # 10
            }
        }
    }
}

// (3)
mod real_impl_of_NetworkClient {
    // (3)
    type NetworkClient = super::RealNetworkClient;

    use super::*;

    impl NetworkClient {
        pub fn new() -> Self {
            NetworkClient {
                /* data here */
            }
        }

        pub fn fetch(&self, a: u32) -> i32 {
            /* does some complicated stuff, maybe network calls */
            # 5
        }
    }
}
#
# fn main() {}
```

This is a bit more complicated than making a mockable struct so let's
go step by step.

1. Returning a real instance

   Because we are only worried about mocking instances of methods, we
can proxy to the real implementation for any associated function
(functions that do not have a receiver, e.g., `&self` or `self:
Rc<Self>`)

   However, because this `new` function returns an instance of the
mockable struct, while the real implementation returns an instance of
the real struct, `RealNetworkClient`, we need to to wrap the
`RealNetworkClient` instance around our mockable `NetworkClient`.

2. Methods

   Methods are fairly simple, we match on ourselves, and then proxy to
the real implementation if we are a real instance or *somehow* get the
mock data if not. More on this *somehow* later.

3. The real implementation

   Like in the mockable struct case, we want to keep our real
implementation *somewhere*, so it can be called when needed. The hitch
is that our real implementation refers to `NetworkClient` assuming it
is the real struct, e.g., when making a new instance, as a return
object or as the name in the `impl` statement. While we could go
through the entire impl block and try to rename every mention of
`NetworkClient` with `RealNetworkClient`, a lazier approach that works
just fine is to use a type alias. However, type aliases are not
allowed inside impl blocks, *yet*. To get around this we put the alias
and the real implementation in its own internal mod.

We have now satisfied the first two bullets of what constitutes a
mockable method.

* By keeping the same function and method signatures, no one from the
  outside looking in can tell that the methods have been transformed.
* The real implementation is saved so it can be called for real
  instances.

However we have not satisfied the third bullet point. There is no way
for the user to provide arbitrary code to be run during tests. We have
a comment saying to just get the fake data, *somehow*. Let's dig in to
how.

### Inject mock methods

Ideally we would like our mocks to be defined per mock instance of our
struct. This would allow us to have two different mock instances of
the same struct, each with their own mocked methods. This means that
the mocked information belongs to the mocked instance. This changes
our definition of our mockable `NetworkClient` from:

```rust
struct NetworkClient(MaybeNetworkClient);

enum MaybeNetworkClient {
    Fake,
    Real(RealNetworkClient),
}

pub struct RealNetworkClient { /* some data */ }
```

to:

```rust
struct NetworkClient(MaybeNetworkClient);

enum MaybeNetworkClient {
    Fake(MockStore),
    Real(RealNetworkClient),
}

pub struct RealNetworkClient { /* some data */ }

#[derive(Default)]
pub struct MockStore { /* store mocks somehow */ }

impl MockStore {
    pub fn get_mock(&self, name: &str) -> Option<Mock> {
        /* somehow return the mock matching the name */
        # None
    }
}

pub struct Mock { /* represent a mock somehow */ }

impl Mock {
    pub fn call<I,O>(self, inputs: I) -> O {
        /* somehow produce an output */
        # panic!()
    }
}
```

We have added a `MockStore` to the `Fake` variant of the
`MaybeNetworkClient` enum. This allows us to store and retrieve mocks
when we have a fake instance of `NetworkClient`.  We derive `Default`
for `MockStore` to denote that it can be created without any
data. This is important because we need to be able to create a mock
instance of the `NetworkClient` from nothing.

We can now now flesh out the mockable `fetch` definition

```rust
impl NetworkClient {
    pub fn fetch(&self, a: u32) -> i32 {
        match self {
            Self(MaybeNetworkClient::Real(real)) => real.fetch(a),
            Self(MaybeNetworkClient::Fake(mock_store)) => {
                mock_store
                    // retrieve the mock using the name of the function
                    .get_mock("fetch")
                    // check the mock was setup; panic if it was not
                    .expect("no mock found for method 'fetch'")
                    // pass in fetch's parameter to the mocked method
                    .call(a)
            }
        }
    }
}
# pub struct NetworkClient(MaybeNetworkClient);
# enum MaybeNetworkClient {
#    Fake(MockStore),
#    Real(RealNetworkClient)
# }
# pub struct RealNetworkClient {}
# impl Mock {
#     pub fn call<I,O>(self, inputs: I) -> O {
#         panic!()
#     }
# }
#
# pub struct MockStore {}
# pub struct Mock {}
#
# impl MockStore {
#     fn get_mock(&self, name: &'static str) -> Option<Mock> {
#         None
#     }
# }
#
# impl RealNetworkClient {
#     pub fn fetch(&self, a: u32) -> i32 {
#         5
#     }
# }
#
# fn main() {}
```

We are now just missing one key piece, saving mocks.

```rust
# pub struct NetworkClient(MaybeNetworkClient);
# enum MaybeNetworkClient {
#    Fake(MockStore),
#    Real(RealNetworkClient)
# }
# pub struct RealNetworkClient {}
# pub struct MockStore {}
#
impl NetworkClient {
    pub fn when_fetch(&mut self) -> When<'_, u32, i32> {
        match &mut self.0 {
            MaybeNetworkClient::Fake(store) => When {
                store,
                method_name: "fetch",
                _marker: std::marker::PhantomData,
            },
            MaybeNetworkClient::Real(_) => panic!("cannot mock a real instance"),
        }
    }
}

// store the expected inputs and output in the type
struct When<'q, I, O> {
    method_name: &'static str,
    store: &'q mut MockStore,
    _marker: std::marker::PhantomData<(*const I, *const O)>,
}

impl<I, O> When<'_, I, O> {
    pub fn then(self, mock: impl FnMut(I) -> O) {
        self.store.save_mock(self.method_name, mock);
    }
}

impl MockStore {
    pub fn save_mock<I,O>(&mut self, name: &'static str, f: impl FnMut(I) -> O) {
        /* somehow save the mock with the given name */
    }
}
```

In the snippet above we have added a `When` struct that allows us to
save information about the method we want to mock prior to the mock
being passed to it. `When` provides a method to that saves the given
mock inside the `MockStore`. We have also added a method to
`NetworkClient` that returns an instance of `When` with information
about the `fetch` method, thus allowing us to mock `fetch`.

This is a simplified version of what `#[faux::methods]` would do to `NetworkClient`.

1. Clones the `impl` block
2. Make the original `impl` block be an `impl` of `RealNetworkClient`
   instead
3. Add `when` methods per public method in the cloned `impl`
4. Modify the cloned methods to either proxy or call the real instance
5. Proxy the associated functions and private methods to the original
   definitions
   
We can now write code that looks like this:

```rust,should_panic
# fn main() {
let mut mock = NetworkClient::fake_please();
mock.when_fetch().then(|i| i as i32);
let fetched = mock.fetch(3);
assert_eq!(fetched, 3);
# }
# struct NetworkClient(MaybeNetworkClient);
# enum MaybeNetworkClient {
#     Fake(MockStore),
#     Real(RealNetworkClient),
# }
# pub struct RealNetworkClient { /* some data */ }
# #[derive(Default)]
# pub struct MockStore { /* store mocks somehow */ }
# impl MockStore {
#     pub fn get_mock(&self, name: &str) -> Option<Mock> {
#         None
#     }
#     pub fn save_mock<I,O>(&mut self, name: &'static str, f: impl FnMut(I) -> O) {
#     }
# }
# pub struct Mock {}
# impl Mock {
#     pub fn call<I,O>(self, inputs: I) -> O {
#         panic!()
#     }
# }
# impl NetworkClient {
#     pub fn fetch(&self, a: u32) -> i32 {
#         match self {
#             Self(MaybeNetworkClient::Real(real)) => real.fetch(a),
#             Self(MaybeNetworkClient::Fake(mock_store)) => {
#                 mock_store
#                     // retrieve the mock using the name of the function
#                     .get_mock("fetch")
#                     // check the mock was setup; panic if it was not
#                     .expect("no mock found for method 'fetch'")
#                     // pass in fetch's parameter to the mocked method
#                     .call(a)
#             }
#         }
#     }
#     fn fake_please() -> NetworkClient {
#          NetworkClient(MaybeNetworkClient::Fake(MockStore::default()))
#     }
#     pub fn when_fetch(&mut self) -> When<'_, u32, i32> {
#         match &mut self.0 {
#             MaybeNetworkClient::Fake(store) => When {
#                 store,
#                 method_name: "fetch",
#                 _marker: std::marker::PhantomData,
#             },
#             MaybeNetworkClient::Real(_) => panic!("cannot mock a real instance"),
#         }
#     }
# }
# struct When<'q, I, O> {
#     method_name: &'static str,
#     store: &'q mut MockStore,
#     _marker: std::marker::PhantomData<(*const I, *const O)>,
# }
# impl<I, O> When<'_, I, O> {
#     pub fn then(self, mock: impl FnMut(I) -> O) {
#         self.store.save_mock(self.method_name, mock);
#     }
# }
# impl RealNetworkClient {
#     pub fn new() -> Self {
#         RealNetworkClient {}
#     }
#     pub fn fetch(&self, a: u32) -> i32 {
#         5
#     }
# }
```

You may have noticed that we have largely omitted the implementation
of `MockStore` and `Mock`. The implementations of these are pretty
hairy, and out of the scope for this blog post, but you may always
read the source code of [`faux`] for more information. In reality,
`MockStore` and `Mock` have quite a bit of complexity, and requires a
few more bounds on the injected mock for safe mocking, while also
having a version with more relaxed bounds that is gated by `unsafe`.

## Final remarks

You have now seen a simplified version of the code `faux`
produces. It's a lot and it is pretty wild but thankfully `faux` will
do it all for you! Remember that this expansion should be gated to
only your `test` thus having no compile nor run time impact to a
`cargo check`, or `cargo build`. If I missed anything or something was
not clear feel free to submit an issue to [`faux`] as the blog also
lives there and I will do my best to explain to correct the blog or
explain something better.

I hope you all get to try `faux`, and tell me what you think abou it!

[release blog post]: ./release.html
[documentation]: https://docs.rs/faux/
[`faux`]: https://github.com/nrxus/faux
