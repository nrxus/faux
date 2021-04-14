# 🪂 faux - landing v0.1 🪂

faux is a library to create mocks out of structs. faux allows you to
mock the methods of structs for testing without complicating or
polluting your code. This post is about the road to a beta version
over the past year. To get started on faux, jump over to its
[repo](https://github.com/nrxus/faux/) and
[documentation](https://docs.rs/faux/)!

## Towards stability

The first release of faux came with this warning:

> faux is in its early alpha stages, so there are no guarantees of API
> stability.

faux has stayed in this stage for over a year, releasing only in the
`0.0.x` range. This allowed faux to experiment and do breaking changes
for the sake of of a better, more usable, API. This was great as an
author as it allowed for a lot of flexibility. However, part of the
usability of a library is its stability. In order to commit to a
(slightly) more stable API, a `0.1` version is being released. The
goal is for faux to still have the flexibility to experiment while
giving users the choice on whether they will take only non-breaking
releases, or if they have the time to upgrade to a release with some
breaking changes.

## What is new

A lot has changed over the past year for faux. In this post let's focus on:

* Argument matchers
* Mocking without closures
* Safe interface

To demonstrate here is some test code from a year ago using faux:

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

There are two major issues with the test above, as far as faux is
concerned.

1. Whenever any argument of a method was a reference, faux only
   allowed mocking it through the use of `unsafe`. This meant that
   even the simplest of mockings required users to wrap their mocks in
   `unsafe`.
2. faux did not have a way to mock only for specific arguments. This
   meant that even for simple cases of pure equality checking, it had
   to be written in a very manual way.
3. faux did not have a way to mock just the return value. This meant
   having to always pass a closure to mock the implementation when in
   reality passing a value would be easier and more ergonomic.

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

We can see how we have now addressed all three of the issues:
1. Mocking methods with references as arguments is no longer unsafe.
2. `when!` now supports passing argument matchers.
3. `then_return!` was added to stub just the return values for the
   simple cases

For more information about the supported argument matchers see the
[docs](https://docs.rs/faux/0.0.10/faux/macro.when.html).

## What is next

While faux is committing to a more stable api, this does not imply
that features will stop coming! As a user of faux, my personal biggest
feature request to myself is mock verification. In general, testing
outputs is always preferred to testing side effects as the latter are
more tied to the implementation. However, there are certain cases
where you really just want a side effect and faux should allow
verifying of those methods. That being said, a lot of the features in
faux today came from people posting issues/PRs, so feel free to look
through the current [issues](https://github.com/nrxus/faux/issues) and
commenting on any that would greatly help your testing experience if
addressed. If there is anything not listed there, please submit an
issue so I can see what other features are missing and needed.

## Contributions

Over the past year faux has had multiple contributors, creating
issues and PRs to help drive the direction of faux and point out any
paper cuts that needed work. A huge thanks to:

* [@muscovite](https://github.com/muscovite)
* [@TedDriggs](https://github.com/TedDriggs)
* [@Wesmania](https://github.com/Wesmania)
* [@sazzer](https://github.com/sazzer)
* [@kungfucop](https://github.com/kungfucop)
* [@audunhalland](https://github.com/audunhalland)
* [@wcampbell0x2a](https://github.com/wcampbell0x2a)
* [@pickfire](https://github.com/pickfire)

for taking your time to contribute to faux.
