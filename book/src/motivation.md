# Motivation

faux was created with the purpose of simplifying mocking in Rust.

## No undue abstractions

A typical technique for mocking in Rust is to use generics to provide
a real implementation in production and a fake implementation in
tests. Adding these generics create a layer of abstraction that is not
necessery outside of tests. Abstractions are not free. Abstractions
affect the readability and maintainability of your code so its cost
needs to be outweighed by its benefits. In this case the benefit is
only testability thus making it an undue burden.

It is the author's belief that writing traits solely for testing are
an undue burden and create an unnecessary layer of abstraction.

In comparison, faux works by changing the implementation of a struct
at compile time. These changes should be gated to only apply during
tests, thus having zero effect in your production code. The goal of
faux is to allow you to create mocks out of your existing code without
forcing you write unnecessary abstractions.

## Mocking behavior

faux is designed to mock *visibile behavior*. In Rust terms, faux is
designed to mock *public methods*. Private methods are not visible and
thus not mockable using faux. Fields of a struct are not behavior and
thus not mockable using faux.

> Free functions and associated functions are behavior but are not
> currently supported by faux. faux's current focus is object mocking,
> but functions may come in the future. Submit an issue if you wish to
> see function mocking so its relative priority can be known.
