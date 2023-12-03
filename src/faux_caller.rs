use std::{ops::Deref, pin::Pin, rc::Rc, sync::Arc};

use crate::{Faux, MaybeFaux};

#[doc(hidden)]
pub trait FauxCaller<R>: Sized {
    fn call<I, O>(self, real_fn: fn(R, I) -> O, fn_name: &'static str, input: I) -> O {
        if let Some(fake) = self.try_as_faux() {
            return unsafe { fake.foo(real_fn, fn_name, input) };
        }
        let real = self
            .try_into_real()
            .expect("faux: bug! Should always be real");
        real_fn(real, input)
    }

    fn try_as_faux(&self) -> Option<&Faux>;
    fn try_into_real(self) -> Option<R>;
}

impl<T, F: FauxCaller<T>> FauxCaller<Box<T>> for Box<F> {
    fn try_as_faux(&self) -> Option<&Faux> {
        let f: &F = self.as_ref();
        f.try_as_faux()
    }

    fn try_into_real(self) -> Option<Box<T>> {
        (*self).try_into_real().map(Box::new)
    }
}

impl<T: Deref, F: FauxCaller<T> + Deref> FauxCaller<Pin<T>> for Pin<F>
where
    T::Target: Sized,
    F::Target: FauxCaller<T::Target>,
{
    fn try_as_faux(&self) -> Option<&Faux> {
        let inner = self.as_ref().get_ref();
        inner.try_as_faux()
    }

    fn try_into_real(self) -> Option<Pin<T>> {
        let inner = unsafe { Pin::into_inner_unchecked(self) };
        inner
            .try_into_real()
            .map(|r| unsafe { Pin::new_unchecked(r) })
    }
}

/* self_type = Owned */

impl<T> FauxCaller<T> for MaybeFaux<T> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<T> {
        match self {
            MaybeFaux::Real(r) => Some(r),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t T> for &'t MaybeFaux<T> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t T> {
        match self {
            MaybeFaux::Real(r) => Some(r),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t mut T> for &'t mut MaybeFaux<T> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t mut T> {
        match self {
            MaybeFaux::Real(r) => Some(r),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Rc<T>> for Rc<MaybeFaux<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Rc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(_) => {
                let inner = Rc::into_inner(self)
                    .expect(&unique_error("Rc<_>", r#"setting `self_type="Rc"`"#));
                let real = match inner {
                    MaybeFaux::Real(r) => r,
                    MaybeFaux::Faux(_) => unreachable!(),
                };

                Some(Rc::new(real))
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Arc<T>> for Arc<MaybeFaux<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Arc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(_) => {
                let inner = Arc::into_inner(self)
                    .expect(&unique_error("Arc<_>", r#"setting `self_type="Arc"`"#));
                let real = match inner {
                    MaybeFaux::Real(r) => r,
                    MaybeFaux::Faux(_) => unreachable!(),
                };

                Some(Arc::new(real))
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

/* self_type = Rc */

impl<T> FauxCaller<T> for MaybeFaux<Rc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<T> {
        match self {
            MaybeFaux::Real(real) => {
                let real = Rc::into_inner(real)
                    .expect(&unique_error("Rc<_>", r#"removing the `self_type="Rc"`"#));
                Some(real)
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t T> for &'t MaybeFaux<Rc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t T> {
        match self {
            MaybeFaux::Real(r) => Some(r.as_ref()),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t mut T> for &'t mut MaybeFaux<Rc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t mut T> {
        match self {
            MaybeFaux::Real(real) => {
                let real = Rc::get_mut(real)
                    .expect(&unique_error("Rc<_>", r#"removing the `self_type="Rc"`"#));
                Some(real)
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Rc<T>> for Rc<MaybeFaux<Rc<T>>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Rc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(r) => Some(Rc::clone(r)),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Arc<T>> for Arc<MaybeFaux<Rc<T>>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Arc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(_) => {
                let inner = Arc::into_inner(self)
                    .expect(&unique_error("Arc<_>", r#"changing to `self_type=Arc"`"#));
                Some(Arc::new(inner.try_into_real().unwrap()))
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

/* self_type = Arc */

impl<T> FauxCaller<T> for MaybeFaux<Arc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<T> {
        match self {
            MaybeFaux::Real(real) => {
                let real = Arc::into_inner(real)
                    .expect(&unique_error("Arc<_>", r#"removing the `self_type="Arc"`"#));
                Some(real)
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t T> for &'t MaybeFaux<Arc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t T> {
        match self {
            MaybeFaux::Real(r) => Some(r.as_ref()),
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<'t, T> FauxCaller<&'t mut T> for &'t mut MaybeFaux<Arc<T>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<&'t mut T> {
        match self {
            MaybeFaux::Real(real) => {
                let real = Arc::get_mut(real)
                    .expect(&unique_error("Arc<_>", r#"removing the `self_type="Arc"`"#));
                Some(real)
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Rc<T>> for Rc<MaybeFaux<Arc<T>>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Rc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(_) => {
                let inner = Rc::into_inner(self)
                    .expect(&unique_error("Rc<_>", r#"changing to `self_type=Rc"`"#));
                Some(Rc::new(inner.try_into_real().unwrap()))
            }
            MaybeFaux::Faux(_) => None,
        }
    }
}

impl<T> FauxCaller<Arc<T>> for Arc<MaybeFaux<Arc<T>>> {
    fn try_as_faux(&self) -> Option<&Faux> {
        match self.as_ref() {
            MaybeFaux::Real(_) => None,
            MaybeFaux::Faux(f) => Some(f),
        }
    }

    fn try_into_real(self) -> Option<Arc<T>> {
        match self.as_ref() {
            MaybeFaux::Real(r) => Some(Arc::clone(r)),
            MaybeFaux::Faux(_) => None,
        }
    }
}

pub(crate) fn unique_error(source: &'static str, suggestion: &'static str) -> String {
    format!("faux tried to get a unique instance from a {source} and failed. Consider {suggestion} argument to both the #[create] and #[method] attributes tagging this struct and its impl.")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Xyz {
        a: i32,
    }

    impl Xyz {
        pub fn new() -> Self {
            Self { a: 1 }
        }

        pub fn by_owned(self, b: i32) -> i32 {
            self.a + b
        }

        pub fn by_ref(&self, b: i32) -> i32 {
            self.a + 2 * b
        }

        pub fn by_mut_ref(&mut self, b: i32) -> i32 {
            self.a + 3 * b
        }

        pub fn by_box(self: Box<Self>, b: i32) -> i32 {
            self.a + 4 * b
        }

        pub fn by_rc(self: Rc<Self>, b: i32) -> i32 {
            self.a + 5 * b
        }

        pub fn by_arc(self: Arc<Self>, b: i32) -> i32 {
            self.a + 6 * b
        }

        pub fn by_pin_ref(self: Pin<&Self>, b: i32) -> i32 {
            self.a + 7 * b
        }

        pub fn by_pin_mut_ref(self: Pin<&mut Self>, b: i32) -> i32 {
            self.a + 8 * b
        }

        pub fn by_pin_box(self: Pin<Box<Self>>, b: i32) -> i32 {
            self.a + 9 * b
        }

        pub fn by_pin_rc(self: Pin<Rc<Self>>, b: i32) -> i32 {
            self.a + 10 * b
        }

        pub fn by_pin_arc(self: Pin<Arc<Self>>, b: i32) -> i32 {
            self.a + 11 * b
        }
    }

    mod owned {
        use crate::{IntoMaybeFaux, MockWrapper};

        use super::*;

        struct WrapperXyz(MaybeFaux<Xyz>);

        unsafe impl MockWrapper for WrapperXyz {
            type Inner = MaybeFaux<Xyz>;

            fn inner(self) -> Self::Inner {
                self.0
            }

            fn wrap(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }

        #[test]
        fn by_owned() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_owned, "", 2), 3);
        }

        #[test]
        fn by_ref() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mock;
            assert_eq!(mock.inner().call(Xyz::by_ref, "", 2), 5);
        }

        #[test]
        fn by_mut_ref() {
            let mut mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mut mock;
            assert_eq!(mock.inner().call(Xyz::by_mut_ref, "", 2), 7);
        }

        #[test]
        fn by_box() {
            let mock: Box<WrapperXyz> = MockWrapper::wrap(Box::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_box, "", 2), 9);
        }

        #[test]
        fn by_rc() {
            let mock: Rc<WrapperXyz> = MockWrapper::wrap(Rc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_rc, "", 2), 11);
        }

        #[test]
        fn by_arc() {
            let mock: Arc<WrapperXyz> = MockWrapper::wrap(Arc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_arc, "", 2), 13);
        }

        #[test]
        fn by_pin_ref() {
            let real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&real);
            let mock: Pin<&WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_ref, "", 2), 15);
        }

        #[test]
        fn by_pin_mut_ref() {
            let mut real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&mut real);
            let mock: Pin<&mut WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_mut_ref, "", 2), 17);
        }

        #[test]
        fn by_pin_box() {
            let mock: Pin<Box<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Box::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_box, "", 2), 19);
        }

        #[test]
        fn by_pin_rc() {
            let mock: Pin<Rc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Rc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_rc, "", 2), 21);
        }

        #[test]
        fn by_pin_arc() {
            let mock: Pin<Arc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Arc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_arc, "", 2), 23);
        }
    }

    mod rc {
        use crate::{IntoMaybeFaux, MockWrapper};

        use super::*;

        struct WrapperXyz(MaybeFaux<Rc<Xyz>>);

        unsafe impl MockWrapper for WrapperXyz {
            type Inner = MaybeFaux<Rc<Xyz>>;

            fn inner(self) -> Self::Inner {
                self.0
            }

            fn wrap(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }

        #[test]
        fn by_owned() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_owned, "", 2), 3);
        }

        #[test]
        fn by_ref() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mock;
            assert_eq!(mock.inner().call(Xyz::by_ref, "", 2), 5);
        }

        #[test]
        fn by_mut_ref() {
            let mut mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mut mock;
            assert_eq!(mock.inner().call(Xyz::by_mut_ref, "", 2), 7);
        }

        #[test]
        fn by_box() {
            let mock: Box<WrapperXyz> = MockWrapper::wrap(Box::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_box, "", 2), 9);
        }

        #[test]
        fn by_rc() {
            let mock: Rc<WrapperXyz> = MockWrapper::wrap(Rc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_rc, "", 2), 11);
        }

        #[test]
        fn by_arc() {
            let mock: Arc<WrapperXyz> = MockWrapper::wrap(Arc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_arc, "", 2), 13);
        }

        #[test]
        fn by_pin_ref() {
            let real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&real);
            let mock: Pin<&WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_ref, "", 2), 15);
        }

        #[test]
        fn by_pin_mut_ref() {
            let mut real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&mut real);
            let mock: Pin<&mut WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_mut_ref, "", 2), 17);
        }

        #[test]
        fn by_pin_box() {
            let mock: Pin<Box<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Box::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_box, "", 2), 19);
        }

        #[test]
        fn by_pin_rc() {
            let mock: Pin<Rc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Rc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_rc, "", 2), 21);
        }

        #[test]
        fn by_pin_arc() {
            let mock: Pin<Arc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Arc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_arc, "", 2), 23);
        }
    }

    mod arc {
        use crate::{IntoMaybeFaux, MockWrapper};

        use super::*;

        struct WrapperXyz(MaybeFaux<Arc<Xyz>>);

        unsafe impl MockWrapper for WrapperXyz {
            type Inner = MaybeFaux<Arc<Xyz>>;

            fn inner(self) -> Self::Inner {
                self.0
            }

            fn wrap(inner: Self::Inner) -> Self {
                Self(inner)
            }
        }

        #[test]
        fn by_owned() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_owned, "", 2), 3);
        }

        #[test]
        fn by_ref() {
            let mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mock;
            assert_eq!(mock.inner().call(Xyz::by_ref, "", 2), 5);
        }

        #[test]
        fn by_mut_ref() {
            let mut mock: WrapperXyz = MockWrapper::wrap(Xyz::new().into_maybe_faux());
            let mock = &mut mock;
            assert_eq!(mock.inner().call(Xyz::by_mut_ref, "", 2), 7);
        }

        #[test]
        fn by_box() {
            let mock: Box<WrapperXyz> = MockWrapper::wrap(Box::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_box, "", 2), 9);
        }

        #[test]
        fn by_rc() {
            let mock: Rc<WrapperXyz> = MockWrapper::wrap(Rc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_rc, "", 2), 11);
        }

        #[test]
        fn by_arc() {
            let mock: Arc<WrapperXyz> = MockWrapper::wrap(Arc::new(Xyz::new()).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_arc, "", 2), 13);
        }

        #[test]
        fn by_pin_ref() {
            let real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&real);
            let mock: Pin<&WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_ref, "", 2), 15);
        }

        #[test]
        fn by_pin_mut_ref() {
            let mut real = Xyz::new().into_maybe_faux();
            let pinned = Pin::new(&mut real);
            let mock: Pin<&mut WrapperXyz> = MockWrapper::wrap(pinned);
            assert_eq!(mock.inner().call(Xyz::by_pin_mut_ref, "", 2), 17);
        }

        #[test]
        fn by_pin_box() {
            let mock: Pin<Box<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Box::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_box, "", 2), 19);
        }

        #[test]
        fn by_pin_rc() {
            let mock: Pin<Rc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Rc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_rc, "", 2), 21);
        }

        #[test]
        fn by_pin_arc() {
            let mock: Pin<Arc<WrapperXyz>> =
                MockWrapper::wrap(Pin::new(Arc::new(Xyz::new())).into_maybe_faux());
            assert_eq!(mock.inner().call(Xyz::by_pin_arc, "", 2), 23);
        }
    }
}
