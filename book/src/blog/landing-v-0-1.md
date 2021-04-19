# 🪂 landing v0.1 🪂

faux is a mocking library that allows you to mock the methods of
structs for testing without complicating or polluting your code. This
post is about the road to a beta version over the past year. To get
started on faux, jump over to its
[repo](https://github.com/nrxus/faux/) and
[documentation](https://docs.rs/faux/)!

## Towards stability

The first release of faux came with this warning:

> faux is in its early alpha stages, so there are no guarantees of API
> stability.

faux has stayed in this stage for over a year, releasing only in the
`0.0.x` range. This allowed faux to experiment and make breaking
changes for the sake of of a better, more usable, API. However, part
of the usability of a library is its stability. The new `0.1` release
marks the beginning of a more stable API. Now, users can choose to
only take non-breaking changes while faux still has the flexibility to
experiment in a more controlled manner.

## What is new

A lot has changed over the past year for faux. This post focuses on:

* Argument matchers
* Mocking without closures
* Safe interface

To demonstrate, here is some test code that uses faux from a year ago:

```rust
#[test]
fn bus_stops() {
    // creates a mock for bus::Client
    let mut bus_client = bus::Client::faux();

    let expected_stops = vec![bus::StopInfo {
        id: String::from("1_1234"),
        direction: String::from("N"),
        name: String::from("some bus"),
        lat: 34.3199,
        lon: 23.12005,
    }];

    // unsafe because mocks with references as inputs required them
    unsafe {
        when!(bus_client.stops).then(|q| {
            // manually assert that it was the expected input
            assert_eq!(
                *q,
                bus::StopsQuery {
                    lat: 34.32,
                    lon: 23.12,
                    lat_span: 0.002,
                    lon_span: 0.0005,
                    max_count: 20,
                }
            );
            // we are always returning the same data so a closure is overkill
            Ok(expected_stops.clone())
        })
    }

     /* snip */
#     let area = Area {
#         lat: 34.32,
#         lon: 23.12,
#         lat_span: 0.002,
#         lon_span: 0.0005,
#         limit: None,
#     };
#
#     let subject = Client::new(seattle_crime::Service::faux(), bus_client);
#
#     let actual_stops = subject
#         .bus_stops(&area)
#         .expect("expected a succesful bus stop response");
#
#     assert_eq!(actual_stops, expected_stops);
}
# fn main() {}
```

Compared to faux today, there are three major issues with the test above:

1. Even the simplest of mocks requires `unsafe`.
2. Checking expected arguments is verbose.
3. No shorthand to mock the return value without a closure.

Now let's look at this same test today:


```rust
#[test]
fn bus_stops() {
    let mut bus_client = bus::Client::faux();

    let expected_stops = vec![bus::StopInfo {
        id: String::from("1_1234"),
        direction: String::from("N"),
        name: String::from("some bus"),
        lat: 34.3199,
        lon: 23.12005,
    }];

    // no more `unsafe` for mocking methods with references as arguments
    // when! supports argument matching
    when!(bus_client.stops(bus::StopsQuery {
        lat: 34.32,
        lon: 23.12,
        lat_span: 0.002,
        lon_span: 0.0005,
        max_count: 20,
    }))
    // for simple cases we can just mock the return value
    .then_return(Ok(expected_stops.clone()));

    /* snip */
#     let area = Area {
#         lat: 34.32,
#         lon: 23.12,
#         lat_span: 0.002,
#         lon_span: 0.0005,
#         limit: None,
#     };
#
#     let subject = Client::new(seattle_crime::Service::faux(), bus_client);
#
#     let actual_stops = subject
#         .bus_stops(&area)
#         .expect("expected a succesful bus stop response");
#
#     assert_eq!(actual_stops, expected_stops);
}
# fn main() {}
```

The three issues mentioned above have all been addressed:
1. Mocking methods with references as arguments is no longer unsafe.
2. `when!` now supports passing argument matchers.
3. `then_return!` was added to mock just the return values for simple
   cases

For more information about the supported argument matchers, see the
[docs](https://docs.rs/faux/0.0.10/faux/macro.when.html).

## What's next

### Guide

As the API surface of faux grows, it has become evident that a
[guide](../faux.html) (WIP) is necessary to cover topics not
appropriate for the API docs. I welcome
[suggestions]((https://github.com/nrxus/faux/issues/38)) on content
that should be covered by the guide.

### Call Verification

Speaking as a user of faux, my personal biggest feature request is
call verification. In general, testing outputs is preferable to
testing side effects, as the latter are more tied to implementation
details. However, there *are* certain cases where you would want to
verify a side effect, so faux should support this.

### Existing issues

A lot of the features that exist in faux today came from people
posting issues/PRs. Please feel free to look through the current
[issues](https://github.com/nrxus/faux/issues) and comment on any that
would greatly help your testing experience if addressed.

### New issues

If you have any feature requests that are not covered by existing
issues, please submit a new issue.

## Contributions

Over the past year, multiple contributors submitted issues and PRs to
help improve and drive the direction of faux. A huge thanks to:

* [@muscovite](https://github.com/muscovite)
* [@TedDriggs](https://github.com/TedDriggs)
* [@Wesmania](https://github.com/Wesmania)
* [@sazzer](https://github.com/sazzer)
* [@kungfucop](https://github.com/kungfucop)
* [@audunhalland](https://github.com/audunhalland)
* [@wcampbell0x2a](https://github.com/wcampbell0x2a)
* [@pickfire](https://github.com/pickfire)

for the time you spent contributing to faux!
