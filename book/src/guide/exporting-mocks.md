# Exporting Mocks Across Crates

As an application or library grows, it is common to split it into
multiple crates.  This separation of responsibilities help to simplify
code, but there is a snag: mocks.

If any code is tagged wth the `#[cfg(test)]` attribute, Rust does not
allow it to be exported outside its crate. This is great! We
definitely do not want to be building or running someone else's tests
when testing our crate. This means, however, that the mockable version
of our structs are also not exported, as `faux` was gated to only work
during tests.

This chapter explores a solution for exporting mocks across
crates. Mocks can then be used within multiple crates of the same
project, or even exposed to users of your library so they can mock
your structs when testing their own library or application.

To better explain, let's start with an example. Let's say we are
building a graphics rendering library, `testable-renderer`. As
expected, `faux` is declared in `dev-dependencies`

```toml
[package]
name = "testable-renderer"

[dev-dependencies]
faux = "^0.1"
```

And the code uses mocks:

```rust
# extern crate faux;
# #[faux::create]
#[cfg_attr(test, faux::create)]
pub struct Renderer {
    /* snip */
    # _inner: u8,
}

# #[faux::methods]
#[cfg_attr(test, faux::methods)]
impl Renderer {
    pub fn new() -> Renderer {
        /* snip */
        # unimplemented!()
    }

    pub fn render(&mut self, texture: &Texture) -> Result<(), RenderError> {
        /* snip */
        # unimplemented!()
    }
}

pub struct Texture;

impl Texture {
    pub fn render(&self, renderer: &mut Renderer) -> Result<(), RenderError> {
        renderer.render(self)
    }
}

#[derive(Debug)]
pub struct RenderError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_textures() {
        let mut renderer = Renderer::faux();
        faux::when!(renderer.render).then(|_| Ok(()));

        let subject = Texture {};
        subject.render(&mut renderer).expect("failed to render the texture")
    }
}
#
# fn main() {
#     let mut renderer = Renderer::faux();
#     faux::when!(renderer.render).then(|_| Ok(()));
#     let subject = Texture {};
#     subject.render(&mut renderer).expect("failed to render the texture")
# }
```

## `faux` as a feature

For the mocks to be exported, they need to be built even outside of
tests. However, we do not want to pollute the production builds of our
library nor anyone using our library with `faux` and mocks, so we make
our dependency on `faux` optional:

```toml
[dependencies]
# set up an optional feature outside of dev-dependencies so that users
# of this library can use our mocks in their own tests
faux = { version = "^0.1", optional = true }

[dev-dependencies]
# our tests still depend on faux; so add it again but do not make it
# optional
faux = "^0.1"
```

Note that we still include `faux` in `dev-dependencies`. Our tests are
always dependent on `faux`, since they use mocks, so the dependency is
not optional.

With this new config, Cargo exposes a new feature flag called
`faux`. `faux` will only be built for tests and when the flag is
enabled, which will be explained later.

## Gating mocks to feature flag

Now that we have a `faux` feature flag, we want our mocks to be
created when that flag is turned on. This is accomplished using the
`any` attribute:

```rust
// mocks are available for both test and the faux feature flag
#[cfg_attr(any(test, feature = "faux"), faux::create)]
pub struct Renderer {
    /* snip */
    # _inner: u8,
}

// mocks are available for both test and the faux feature flag
#[cfg_attr(any(test, feature = "faux"), faux::methods)]
impl Renderer {
    pub fn new() -> Renderer {
        /* snip */
        # unimplemented!()
    }

    pub fn render(&mut self, texture: &Texture) -> Result<(), RenderError> {
        /* snip */
        # unimplemented!()
    }
}

# pub struct RenderError;
# pub struct Texture;
# fn main() {}
```

The key thing to remember here is replacing:

```rust
#[cfg_attr(test, ...)]
# fn main()
```

with

```rust
#[cfg_attr(any(test, feature = "faux"), ...)]
# fn main()
```

This tells Rust to use the `faux` attributes (`create` and `methods`)
for either `test` or the `faux` feature flag. You can learn more about
the `any` attribute in the [Rust Reference].

These are all the changes necessary in our rendering library. The
tests remain the same, and there are no implementation changes.


## Using the `faux` feature

Let's now move on to a dependent of our rendering library. The
dependency is marked in its `Cargo.toml` as:

```toml
[dependencies]
testable-renderer = * // some version
```

We would now like to use `testable-renderer` to render multiple
textures for some `World` struct:

```rust
# mod testable_renderer {
#     extern crate faux;
#     #[faux::create]
#     pub struct Renderer {
#         _inner: u8,
#     }
#     #[faux::methods]
#     impl Renderer {
#         pub fn new() -> Renderer {
#             todo!()
#         }
#         pub fn render(&mut self, texture: &Texture) -> Result<(), RenderError> {
#             todo!()
#         }
#     }
#     pub struct Texture;
#     impl Texture {
#         pub fn render(&self, renderer: &mut Renderer) -> Result<(), RenderError> {
#             renderer.render(self)
#         }
#     }
#     #[derive(Debug)]
#     pub struct RenderError;
# }
use testable_renderer::{RenderError, Renderer, Texture};

struct World {
    player: Texture,
    enemy: Texture,
}

impl World {
    pub fn new() -> Self {
        World {
            player: Texture {},
            enemy: Texture {},
        }
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<(), RenderError> {
        self.player.render(renderer)?;
        self.enemy.render(renderer)?;

        Ok(())
    }
}
# fn main() {}
```

We would like to write tests for our `World::render` method, but since
rendering is an expensive opration, this is hard to do without
mocks. Thankfully, `testable-renderer` is set up to expose its mocks,
so we activate them by configuring the feature flag in our
`Cargo.toml`:

```toml
[package]
# important so features turned on by dev-dependencies don't infect the
# binary when doing a normal build. This lets us have different feature
# flags in dev-dependencies vs normal dependencies.
resolver = "2"

[dependencies]
# our normal dependency does not activate `faux`, thus keeping it out of
# our released binary
testable-renderer = * # some version

[dev-dependencies]
# for tests, we activate the `faux` feature in our dependency so that
# we can use the exposed mocks
testable-renderer = { version = "*", features = ["faux"] }

# still depend on `faux` so we can use setup the mocks
faux = "^0.1"
```

The important takeaways are:
* `resolver = "2"`. This is needed so `faux` stays out of our normal
  builds. See the [Cargo Reference].

* Turn on the feature flag under `[dev-dependencies]`. We only want to
  have access to the mocks in `testable-renderer` when building tests.

We can now write tests as per usual:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_world() {
        // the test target enables the faux feature on `testable-renderer`
        // thus allowing us to use the mocks of the *external* crate
        let mut renderer = Renderer::faux();
        faux::when!(renderer.render).then(|_| Ok(()));

        let world = World::new();
        world.render(&mut renderer).expect("failed to render the world")
    }
}

# fn main() {}
```

## Recap

The library that wants to export its mocks needs to:

* Add `faux` as an optional dependency in `Cargo.toml`.

* Create the mocks not only during tests, but also when the `faux`
  feature flag is turned on.

The library or application that wants to use the exported mocks needs
to:

* Change the feature resolver to "2" in its `Cargo.toml`. Be aware
  that if you are using a workspace, this needs to be changed in the
  workspace's `Cargo.toml`.

* Add the dependency with the exported mocks under `dev-dependencies`
  with the `faux` flag enabled in `Cargo.toml`.

To see this in action, take a look at the example in the [faux
repository]. `testable-renderer` is the library with the exported
mocks and `world-renderer` is the application that uses these mocks.

[Rust Reference]: https://doc.rust-lang.org/reference/conditional-compilation.html
[Cargo Reference]: https://doc.rust-lang.org/nightly/cargo/reference/features.html#feature-resolver-version-2
[faux repository]: https://github.com/nrxus/faux/tree/master/examples
