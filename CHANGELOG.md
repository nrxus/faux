# CHANGELOG

## v0.0.3
*  Async methods may now be mocked
   * [tests](/tests/asynchronous.rs)
* Mocked structs may now contain generics
   * [tests](/tests/generic_struct.rs)
* Mocked impl blocks may now contain a path
   * Paths that contain `super` or `crate` have to use the
     `faux::methods(path)` syntax.
   * [tests](/tests/paths.rs)

## v0.0.2

* Made mocked structs respect Sync+Send+Debug.
   * This allows mocking structs that are enforced to be
     Sync/Send/Debug. This has a performance impact as we now used
     Mutex rather than RefCell for interor mutability. This is unideal
     and will be addressed in a future release.

### Breaking Change

* All closures used for mocking methods must now implement Send. This
  means that all of its captures variables need to implement
  Send. This is unideal but needed for now to be able to mock things
  like Rocket's request guards.

## v0.0.1

* Initial alpha release
