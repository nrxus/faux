# Getting Started

`faux` makes liberal use of unsafe Rust feature, so it is only
recommended for use inside tests. Follow the steps below to configure
`faux` and the created mocks to only exist during tests.

## Installation

`faux` should be added under `[dev-dependencies]` in `Cargo.toml`.

```toml
[dev-dependencies]
faux = "^0.1"
```

This makes sure that `faux` only gets included when compiling and
running tests, thus making it impossible to leak into production code.

## Your First Mock

`faux` is able to mock a `struct` and its public methods. To do this,
`faux` provides two attributes: `#[faux::create]` and
`#[faux::methods]`. `#[faux::create]` tags the struct we wish to make
mockable. `#[faux::methods]` tags the `impl` blocks of that
struct. Both of these attributes must be used.

```rust
#[cfg_attr(test, faux::create)]
pub struct MyStructToMock { /* fields */ }

#[cfg_attr(test, faux::methods)]
impl MyStructToMock { /* methods to mock */ }
# fn main() {}
```

### Example

Let's say you are writing a restaurant reservation system. One of the
core structs in this system is a `RestaurantClient` which sends HTTP
requests to get availability times for a restaurant, create a
reservation, cancel, etc.

```rust
pub struct RestaurantClient {
    /* snip */
}

impl RestaurantClient {
    pub fn new() -> Self {
        /* snip */
        # todo!()
    }

    pub fn availabilities(&self) -> Result<Vec<Availability>, Error> {
        /* GET to some HTTP endpoint */
        # todo!()
    }

    pub fn reserve(&self, availability: Availability) -> Result<Reservation, Error> {
        /* POST to some HTTP endpoint */
        # todo!()
    }

    pub fn cancel(&self, reservation: Reservation) -> Result<(), Error> {
        /* DELETE to some HTTP endpoint */
        # todo!()
    }
}
#
# pub struct Reservation { /* snip */ }
# pub struct Availability { /* snip */ }
# pub struct Error { /* snip */ }
# fn main() {}
```

This type is not interesting to unit-test in itself as it is very
declarative. Aside from the fact that it doesn't have any real logic
to unit-test, calling these methods will send actual HTTP requests to
create or cancel reservations, which is bound to make your tests slow
and flaky. You will also probably have some really angry restaurants.

> You may want to have some kind of integration or enemy tests that
> verifies the overall correctness of your service that will end up
> testing this struct, but that goes beyond the scope of this guide.

However, a more interessting part of your library deals with
*choosing* from the possible availabilities and reserves a spot at the
restaurant. Let's call it `Concierge`.

```rust
use restaurant_client::{RestaurantClient, Reservation};

pub struct Concierge {
    client: RestaurantClient,
}

impl Concierge {
    pub fn new(client: RestaurantClient) -> Self {
        Concierge {
            client,
        }
    }

    pub fn reserve_matching(&self, options: Options) -> Result<Reservation, Error> {
        /* logic to find a matching availability and reserve it */
        # todo!()
    }
}
#
# pub struct Options { /* snip */ }
# pub enum Error {
#     Client(restaurant_client::Error),
# }
# impl From<restaurant_client::Error> for Error {
#     fn from(error: restaurant_client::Error) -> Self {
#         Error::Client(error)
#     }
# }
# mod restaurant_client {
#     pub struct RestaurantClient {}
#     impl RestaurantClient {
#        pub fn availabilities(&self) -> Result<Vec<Availability>> {
#            todo!()
#        }
#        pub fn reserve(&self, availability: Availability) -> Result<Reservation> {
#            todo!()
#        }
#     }
#     pub struct Reservation { /* snip */ }
#     pub struct Availability { /* snip */ }
#     pub struct Error { /* snip */ }
#     pub type Result<T> = std::result::Result<T, Error>;
# }
# fn main() {}
```

Unlike `ReservationClient`, the `Concierge` does hold a key piece of
domain logic: how to choose between the available times. This logic is
worth unit tests as it is vital to our service and we want to make
sure that it continues working as we refactor or add features to
`reserve_matching`. However, we do not want to make actual calls to
the `ReservationClient` as that would mean having to make network
requests. To solve this, we decide to mock `ReservationClient` for our
tests. `faux` makes it easy to make this struct mockable using the
`faux::create` and `faux::methods` attributes.

```rust
// gate the attribute to only tests
// `faux` is (and should be!) only available when running tests
#[cfg_attr(test, faux::create)]
pub struct RestaurantClient {
    /* snip */
}

// gate the attribute to only tests
#[cfg_attr(test, faux::methods)]
impl RestaurantClient {
    pub fn new() -> Self {
        /* snip */
        # todo!()
    }

    pub fn availabilities(&self) -> Result<Vec<Availability>, Error> {
        /* snip */
        # todo!()
    }

    pub fn reserve(&self, availability: Availability) -> Result<Reservation, Error> {
        /* snip */
        # todo!()
    }

    pub fn cancel(&self,  reservation: Reservation) -> Result<(), Error> {
        /* snip */
        # todo!()
    }
}
#
# pub struct Reservation { /* snip */ }
# pub struct Availability { /* snip */ }
# pub struct Error { /* snip */ }
# fn main() {}
```

Using these two attributes allows/signals `faux` to hook into the
struct and its methods at compile time to create mockable versions of
them that can be used in your tests. Note that there are zero changes
to the implementation or signature of `ReservationClient`, the only
change is tagging it with the `faux` attributes.

```rust
# use restaurant_client::{RestaurantClient, Reservation};
# pub struct Concierge {
#     client: RestaurantClient,
# }
# impl Concierge {
#     pub fn new(client: RestaurantClient) -> Self {
#         Concierge {
#             client,
#         }
#     }
#     pub fn reserve_matching(&self, options: Options) -> Result<Reservation, Error> {
#         let _ = options;
#         let chosen_availability = self.client
#             .availabilities()?
#             .pop()
#             .ok_or(Error::NoReservations)?;
#         let reservation = self.client.reserve(chosen_availability)?;
#         Ok(reservation)
#     }
# }
#
# pub struct Options { /* snip */ }
# #[derive(Clone, Debug)]
# pub enum Error {
#     Client(restaurant_client::Error),
#     NoReservations,
# }
# impl From<restaurant_client::Error> for Error {
#     fn from(error: restaurant_client::Error) -> Self {
#         Error::Client(error)
#     }
# }
# mod restaurant_client {
#     #[faux::create]
#     pub struct RestaurantClient {}
#     #[faux::methods]
#     impl RestaurantClient {
#        pub fn availabilities(&self) -> Result<Vec<Availability>> {
#            todo!()
#        }
#        pub fn reserve(&self, availability: Availability) -> Result<Reservation> {
#            todo!()
#        }
#     }
#     #[derive(Clone, Debug, PartialEq)]
#     pub struct Reservation { /* snip */ }
#     #[derive(Clone, Debug, PartialEq)]
#     pub struct Availability { /* snip */ }
#     #[derive(Clone, Debug)]
#     pub struct Error { /* snip */ }
#     pub type Result<T> = std::result::Result<T, Error>;
# }
# extern crate faux;
# use faux::when;
# use restaurant_client::Availability;
# fn main() {
#     // first test
#     let mut client = RestaurantClient::faux();
#     let availability = Availability { /*snip */ };
#     let expected_reservation = Reservation { /* snip */ };
#
#     when!(client.availabilities())
#         .then_return(Ok(vec![availability.clone()]));
#
#     when!(client.reserve(availability))
#         .then_return(Ok(expected_reservation.clone()));
#
#     let subject = Concierge::new(client);
#     let options = Options { /* snip */ };
#     let reservation = subject
#         .reserve_matching(options).expect("expected successful reservation");
#
#     assert_eq!(reservation, expected_reservation);
#
#     // second test
#     let mut client = RestaurantClient::faux();
#     when!(client.availabilities()).then_return(Ok(vec![]));
#
#     let subject = Concierge::new(client);
#     let options = Options { /* snip */ };
#     let error = subject
#         .reserve_matching(options)
#         .expect_err("expected error reservation");
#
#     assert!(matches!(error, Error::NoReservations));
# }
#
#[cfg(test)]
mod tests {
    use super::*;

    use faux::when;

    #[test]
    fn selects_the_only_one() {
        // A `faux()` function to every mockable struct
        // to instantiate a mock instance
        let mut client = RestaurantClient::faux();
        let availability = Availability { /*snip */ };
        let expected_reservation = Reservation { /* snip */ };

        // when!(...) lets you stub the return method of the mock struct
        when!(client.availabilities())
            .then_return(Ok(vec![availability.clone()]));

        // when!(...) lets you specify expected arguments
        // so only invocations that match that argument return the stubbed data
        when!(client.reserve(availability))
            .then_return(Ok(expected_reservation.clone()));

        let subject = Concierge::new(client);
        let options = Options { /* snip */ };
        let reservation = subject
            .reserve_matching(options)
            .expect("expected successful reservation");

        assert_eq!(reservation, expected_reservation);
    }

    #[test]
    fn fails_when_empty() {
        let mut client = RestaurantClient::faux();
        when!(client.availabilities()).then_return(Ok(vec![]));

        let subject = Concierge::new(client);
        let options = Options { /* snip */ };
        let error = subject
            .reserve_matching(options)
            .expect_err("expected error reservation");

        assert!(matches!(error, Error::NoReservations));
    }
}
```

You have now successfully added tests for `Concierge` that use a mock
instance of the `RestaurantClient`. Note that neither the
implementation of `Concierge` nor `RestaurantClient` had to change in
order to be mockable. You can write production ready code without
incurring any abstraction penalty for using mocks in testing.

## Recap

* Use `faux` as a `dev-dependency` to avoid it leaking into production
  code.

* `faux::create` and `faux::methods` are attributes used to tag
  structs and methods for mocking. These tags should be gated to tests
  only using `#[cfg_attr(test, ...)]`

* `faux::when!` is used to stub the returned data of a method in a
  mocked struct.

* `faux::when!` lets you specify argument matchers so mocks are used
  only for certain invocations. The default is an equality matcher,
  but there are also other matchers if you want to match any argument,
  match a pattern, or match based on the result of a given predicate.
  See the [when docs] for more information.

[matcher docs]: https://docs.rs/faux/0.1.3/faux/macro.when.html
