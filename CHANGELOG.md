# CHANGELOG

## UPCOMING
* Better macro error messages
  * [PR](https://github.com/nrxus/faux/pull/23). Thanks a lot [@TedDriggs](https://github.com/TedDriggs)!
* `self: Box<Self>`, `self: Rc<Self>`, and `self: Arc<Self>` are now
  allowed in methods inside an impl blocked tagged by
  `#[methods]`. See the tests for what is possible with arbitrary self
  types.
  * [tests](/tests/arbitrary_self.rs)

### Breaking Change

* Specifying a path in `#[methods]` has changed from:
  `#[methods(path::to::mod)]` to `#[methods(path =
  "path::to::mod")]`. This is done to allow further current and future
  arguments to be passed to the attribute.

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
