use faux;
use std::{boxed::Box, rc::Rc, sync::Arc};

#[faux::create]
pub struct Owned {}

#[faux::methods]
impl Owned {
    pub fn new() -> Self {
        Owned {}
    }

    //normal receivers
    pub fn by_ref(&self) {}
    pub fn by_mut_ref(&mut self) {}
    pub fn by_value(self) {}
    #[allow(unused_mut)]
    pub fn by_mut_value(mut self) {}

    //self with a type
    pub fn by_value2(self: Self) {}
    pub fn by_ref2(self: &Self) {}
    pub fn by_mut_ref2(self: &mut Self) {}
    #[allow(unused_mut)]
    pub fn by_mut_value2(mut self: Self) {}
    pub fn by_box(self: std::boxed::Box<Self>) {}
}

#[faux::create(self_type = "Rc")]
pub struct ByRc {}

#[faux::methods(self_type = "Rc")]
impl ByRc {
    pub fn new() -> Self {
        ByRc {}
    }

    pub fn new_rc() -> Rc<Self> {
        Rc::new(ByRc {})
    }

    pub fn new_rc2() -> Rc<ByRc> {
        Rc::new(ByRc {})
    }

    pub fn by_rc(self: Rc<Self>) {}
    pub fn by_rc2(self: std::rc::Rc<ByRc>) {}
    pub fn by_ref(&self) {}
}

#[faux::create(self_type = "Arc")]
pub struct ByArc {}

#[faux::methods(self_type = "Arc")]
impl ByArc {
    pub fn new() -> ByArc {
        ByArc {}
    }

    pub fn new_arc() -> Arc<Self> {
        Arc::new(ByArc {})
    }

    pub fn new_arc2() -> Arc<ByArc> {
        Arc::new(ByArc {})
    }

    pub fn by_arc(self: Arc<Self>) {}
    pub fn by_arc2(self: std::sync::Arc<ByArc>) {}
    pub fn by_ref(&self) {}
}

#[faux::create(self_type = "Box")]
pub struct ByBox {}

#[faux::methods(self_type = "Box")]
impl ByBox {
    pub fn new() -> ByBox {
        ByBox {}
    }

    pub fn new_box() -> Box<Self> {
        Box::new(ByBox {})
    }

    pub fn new_box2() -> Box<ByBox> {
        Box::new(ByBox {})
    }

    pub fn by_box(self: Box<Self>) {}
    pub fn by_box2(self: std::boxed::Box<ByBox>) {}
    pub fn by_ref(&self) {}
    pub fn by_mut_ref(&mut self) {}
    pub fn by_value(self) {}
}
