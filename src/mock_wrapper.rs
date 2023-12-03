use std::{ops::Deref, pin::Pin, rc::Rc, sync::Arc};

#[doc(hidden)]
/// Signifies that it is transparent wrapper around a mock struct
///
/// # Safety
///
/// This trait is only ever safe to implement if transmuting between
/// `MockWrapper` and `MockWrapper::Inner` is guaranteed to be
/// safe. In practice this means that `MockWrapper` is a single field
/// struct (e.g., a one element tuple) that wraps inner and has a
/// `#[repr(transparent)]` directive.
pub unsafe trait MockWrapper: Sized {
    /// The type we are transparently wrapping
    type Inner;

    /// Gets the inner type
    fn inner(self) -> Self::Inner;

    /// Wraps the inner type
    fn wrap(inner: Self::Inner) -> Self;
}

unsafe impl<'m, M: MockWrapper> MockWrapper for &'m M {
    type Inner = &'m M::Inner;

    fn inner(self) -> Self::Inner {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(self) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(inner) }
    }
}

unsafe impl<'m, M: MockWrapper> MockWrapper for &'m mut M {
    type Inner = &'m mut M::Inner;

    fn inner(self) -> Self::Inner {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(self) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(inner) }
    }
}

unsafe impl<M: MockWrapper> MockWrapper for Rc<M> {
    type Inner = Rc<M::Inner>;

    fn inner(self) -> Self::Inner {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(self) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(inner) }
    }
}

unsafe impl<M: MockWrapper> MockWrapper for Arc<M> {
    type Inner = Arc<M::Inner>;

    fn inner(self) -> Self::Inner {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(self) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(inner) }
    }
}

unsafe impl<M: MockWrapper> MockWrapper for Box<M> {
    type Inner = Box<M::Inner>;

    fn inner(self) -> Self::Inner {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(self) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: transmute safety guarantee by M: MockWrapper
        unsafe { std::mem::transmute(inner) }
    }
}

unsafe impl<M: MockWrapper + Deref> MockWrapper for Pin<M>
where
    M::Inner: Deref,
{
    type Inner = Pin<M::Inner>;

    fn inner(self) -> Self::Inner {
        // Safety: we are going to re-pin it without any moving
        let wrapper = unsafe { Pin::into_inner_unchecked(self) };
        let inner = wrapper.inner();
        unsafe { Pin::new_unchecked(inner) }
    }

    fn wrap(inner: Self::Inner) -> Self {
        // Safety: we are going to re-pin it without any moving
        let inner = unsafe { Pin::into_inner_unchecked(inner) };
        let wrapper = MockWrapper::wrap(inner);
        unsafe { Pin::new_unchecked(wrapper) }
    }
}

unsafe impl<M: MockWrapper> MockWrapper for Option<M> {
    type Inner = Option<M::Inner>;

    fn inner(self) -> Self::Inner {
        self.map(|m| m.inner())
    }

    fn wrap(inner: Self::Inner) -> Self {
        inner.map(|i| MockWrapper::wrap(i))
    }
}

unsafe impl<M: MockWrapper, E> MockWrapper for Result<M, E> {
    type Inner = Result<M::Inner, E>;

    fn inner(self) -> Self::Inner {
        self.map(|m| m.inner())
    }

    fn wrap(inner: Self::Inner) -> Self {
        inner.map(|i| MockWrapper::wrap(i))
    }
}
