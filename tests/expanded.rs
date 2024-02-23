use std::pin::{pin, Pin};

use futures::Future;

pub struct Foo(pub faux::MaybeFaux<_FauxOriginal_Foo>);

impl Foo {
    pub fn faux() -> Self {
        Self(::faux::MaybeFaux::faux("Foo"))
    }
}
unsafe impl ::faux::MockWrapper for Foo {
    type Inner = ::faux::MaybeFaux<_FauxOriginal_Foo>;
    fn inner(self) -> Self::Inner {
        self.0
    }
    fn wrap(inner: Self::Inner) -> Self {
        Self(inner)
    }
}
#[allow(non_camel_case_types)]
pub struct _FauxOriginal_Foo {}

impl Foo {
    pub async fn new() -> Self {
        Self(faux::MaybeFaux::Real(<_FauxOriginal_Foo>::new().await))
    }
    pub async fn associated() -> u32 {
        <_FauxOriginal_Foo>::associated().await
    }
    pub async fn fetch(&self) -> i32 {
        {
            use ::faux::FauxCaller;
            use ::faux::MockWrapper;
            let inner = self.inner();
            inner.call(<_FauxOriginal_Foo>::_faux_fetch, stringify!(fetch), ()).await
        }
    }
    async fn private(&self) -> i32 {
        {
            let wrapped = ::faux::MockWrapper::inner(self);
            let _maybe_faux_real = match::faux::FauxCaller::try_into_real(wrapped){
        Some(r) => r,
        None => panic!("faux error: private methods are not stubbable; and therefore not directly callable in a mock"),

        };
            <_FauxOriginal_Foo>::private(_maybe_faux_real).await
        }
    }
}
impl Foo {
    pub fn _when_fetch<'_faux_implicit_ref, '_faux_mock_lifetime>(
        &'_faux_mock_lifetime mut self,
    ) -> faux::When<
        '_faux_mock_lifetime,
        &'_faux_implicit_ref _FauxOriginal_Foo,
        (),
        Pin<Box<dyn Future<Output = i32> + '_faux_implicit_ref + Send>>,
        faux::matcher::AnyInvocation,
    > {
        match &mut self.0 {
            faux::MaybeFaux::Faux(_maybe_faux_faux) => {
                faux::When::new(<_FauxOriginal_Foo>::_faux_fetch, "fetch", _maybe_faux_faux)
            }
            faux::MaybeFaux::Real(_) => panic!("not allowed to stub a real instance!"),
        }
    }
}
mod _faux_real_impl__FauxOriginal_Foo_f8735cebdbab45e49a5a80d951670009 {
    #[allow(non_camel_case_types)]
    #[allow(clippy::builtin_type_shadow)]
    use super::_FauxOriginal_Foo as Foo;
    use super::*;
    impl Foo {
        pub async fn new() -> Self {
            Foo {}
        }
        pub async fn associated() -> u32 {
            5
        }
        pub async fn fetch(&self) -> i32 {
            self.private().await
        }
        pub(super) async fn private(&self) -> i32 {
            3
        }
    }
    impl _FauxOriginal_Foo {
        pub fn _faux_fetch<'s>(&'s self, (): ()) -> Pin<Box<dyn Future<Output = i32> + Send + 's>> {
            Box::pin(<_FauxOriginal_Foo>::fetch(self))
        }
    }
}

#[test]
fn test() {
    let mut foo = Foo::faux();
    faux::when!(foo.fetch).once().then_return(Box::pin(x()));
}

async fn x() -> i32 {
    4
}
