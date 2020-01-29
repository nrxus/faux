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

    // the following two will compile
    // but they will only work if self is the *only*
    // reference to the object. Otherwise they will panic
    // with a message about using the self_type argument instead
    pub fn by_rc(self: Rc<Self>) {}
    pub fn by_arc(self: Arc<Self>) {}
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

#[test]
fn by_rc_from_owned() {
    // getting and invoking real instances/methods works
    let real_rcd = Rc::new(Owned::new());
    real_rcd.by_rc();

    // mocking also works BUT
    // mocks need a `&mut` when mocking a method
    // so prepare the mock before wrapping it around an Rc
    let mut faux_owned = Owned::faux();
    faux::when!(faux_owned.by_rc).safe_then(|_| {});

    let faux_rcd = Rc::new(faux_owned);
    faux_rcd.by_rc();
}

#[test]
#[should_panic]
fn by_rc_from_owned_panics_if_cloned() {
    let rcd = Rc::new(Owned::new());
    let clone = rcd.clone();
    // panics because faux cannot get the owned value from the Rc.
    clone.by_rc();
    // rcd.by_rc(); would also have panicked
}

#[test]
fn by_rc() {
    let rcd = Rc::new(ByRc::new());
    let clone = rcd.clone();

    // cloning the Rc works when self_type = Rc was specified in the mock
    clone.by_rc();
    rcd.by_rc();

    // or get it already wrapped
    let rcd = ByRc::new_rc();
    rcd.by_rc();

    // mocking must be done prior to wrapping it in an Rc
    let mut owned = ByRc::faux();
    faux::when!(owned.by_rc).safe_then(|_| {});

    let rcd = Rc::new(owned);
    rcd.by_rc();
}

#[test]
fn by_box_from_owned() {
    let real_boxed = Box::new(Owned::new());
    real_boxed.by_box();

    // can be boxed right away because a &mut can be obtained from a Box
    let mut faux_boxed = Box::new(Owned::faux());
    faux::when!(faux_boxed.by_box).safe_then(|_| {});
    faux_boxed.by_box();
}

#[test]
fn by_box() {
    let real_boxed = ByBox::new_box();
    real_boxed.by_box();

    // can be boxed right away because a &mut can be obtained from a Box
    let mut faux_boxed = Box::new(ByBox::faux());
    faux::when!(faux_boxed.by_box).safe_then(|_| {});
    faux_boxed.by_box();
}
