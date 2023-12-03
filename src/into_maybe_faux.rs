use std::{ops::Deref, pin::Pin, rc::Rc, sync::Arc};

use crate::{faux_caller::unique_error, MaybeFaux};

#[doc(hidden)]
pub trait IntoMaybeFaux<Out> {
    fn into_maybe_faux(self) -> Out;
}

impl<O, T: IntoMaybeFaux<O>> IntoMaybeFaux<Box<O>> for Box<T> {
    fn into_maybe_faux(self) -> Box<O> {
        Box::new((*self).into_maybe_faux())
    }
}

impl<O, T: IntoMaybeFaux<O>, E> IntoMaybeFaux<Result<O, E>> for Result<T, E> {
    fn into_maybe_faux(self) -> Result<O, E> {
        self.map(T::into_maybe_faux)
    }
}

impl<O, T: IntoMaybeFaux<O>> IntoMaybeFaux<Option<O>> for Option<T> {
    fn into_maybe_faux(self) -> Option<O> {
        self.map(T::into_maybe_faux)
    }
}

impl<O: Deref, T: IntoMaybeFaux<O> + Deref> IntoMaybeFaux<Pin<O>> for Pin<T> {
    fn into_maybe_faux(self) -> Pin<O> {
        let real = unsafe { Pin::into_inner_unchecked(self) };
        unsafe { Pin::new_unchecked(real.into_maybe_faux()) }
    }
}

/* owned */

impl<T> IntoMaybeFaux<MaybeFaux<T>> for T {
    fn into_maybe_faux(self) -> MaybeFaux<T> {
        MaybeFaux::Real(self)
    }
}

impl<T> IntoMaybeFaux<Rc<MaybeFaux<T>>> for Rc<T> {
    fn into_maybe_faux(self) -> Rc<MaybeFaux<T>> {
        let real =
            Rc::into_inner(self).expect(&unique_error("Rc<_>", r#"setting `self_type="Rc"`"#));
        Rc::new(MaybeFaux::Real(real))
    }
}

impl<T> IntoMaybeFaux<Arc<MaybeFaux<T>>> for Arc<T> {
    fn into_maybe_faux(self) -> Arc<MaybeFaux<T>> {
        let real =
            Arc::into_inner(self).expect(&unique_error("Arc<_>", r#"setting `self_type="Arc"`"#));
        Arc::new(MaybeFaux::Real(real))
    }
}

/* self_type = Rc */

impl<T> IntoMaybeFaux<MaybeFaux<Rc<T>>> for T {
    fn into_maybe_faux(self) -> MaybeFaux<Rc<T>> {
        MaybeFaux::Real(Rc::new(self))
    }
}

impl<T> IntoMaybeFaux<Rc<MaybeFaux<Rc<T>>>> for Rc<T> {
    fn into_maybe_faux(self) -> Rc<MaybeFaux<Rc<T>>> {
        Rc::new(MaybeFaux::Real(self))
    }
}

impl<T> IntoMaybeFaux<Arc<MaybeFaux<Rc<T>>>> for Arc<T> {
    fn into_maybe_faux(self) -> Arc<MaybeFaux<Rc<T>>> {
        let real = Arc::into_inner(self)
            .expect(&unique_error("Arc<_>", r#"changing to `self_type="Arc"`"#));
        Arc::new(MaybeFaux::Real(Rc::new(real)))
    }
}

/* self_type = Arc */

impl<T> IntoMaybeFaux<MaybeFaux<Arc<T>>> for T {
    fn into_maybe_faux(self) -> MaybeFaux<Arc<T>> {
        MaybeFaux::Real(Arc::new(self))
    }
}

impl<T> IntoMaybeFaux<Rc<MaybeFaux<Arc<T>>>> for Rc<T> {
    fn into_maybe_faux(self) -> Rc<MaybeFaux<Arc<T>>> {
        let real =
            Rc::into_inner(self).expect(&unique_error("Rc<_>", r#"changing to `self_type="Rc"`"#));
        Rc::new(MaybeFaux::Real(Arc::new(real)))
    }
}

impl<T> IntoMaybeFaux<Arc<MaybeFaux<Arc<T>>>> for Arc<T> {
    fn into_maybe_faux(self) -> Arc<MaybeFaux<Arc<T>>> {
        Arc::new(MaybeFaux::Real(self))
    }
}
