#[faux::create]
pub struct Foo {}

pub trait MyTrait {}
pub struct MyStruct {}
impl MyTrait for MyStruct {}

// #[faux::methods]
impl Foo {
    pub fn foo(&self, _: impl Fn(i32) -> i32) -> i32 {
        todo!()
    }

    pub fn bar(&self, a: i32) -> impl Fn(i32) -> f64 {
        match self {
            Foo(faux::MaybeFaux::Real(r)) => r.bar(a),
            Foo(faux::MaybeFaux::Faux(f)) => {
                let mut f = f.try_lock().unwrap();
                unsafe { f.get_mock("bar").unwrap().call(a) }
            }
        }
    }

    pub fn other(&self, a: f64) -> impl MyTrait {
        match self {
            Foo(faux::MaybeFaux::Real(r)) => r.other(a),
            Foo(faux::MaybeFaux::Faux(f)) => {
                let mut f = f.try_lock().unwrap();
                unsafe { f.get_mock("other").unwrap().call(a) }
            }
        }
    }

    pub fn _when_bar<O: Fn(i32) -> f64 + 'static>(
        &mut self,
        f: impl FnMut(i32) -> O + Send + 'static,
    ) {
        match &mut self.0 {
            faux::MaybeFaux::Real(_) => panic!("ahhh"),
            faux::MaybeFaux::Faux(mock_store) => {
                let when = faux::When::new("bar", mock_store.get_mut().unwrap());
                when.safe_then(f);
            }
        }
    }

    pub fn _when_other<O: MyTrait + 'static>(&mut self, f: impl FnMut(i32) -> O + Send + 'static) {
        match &mut self.0 {
            faux::MaybeFaux::Real(_) => panic!("ahhh"),
            faux::MaybeFaux::Faux(mock_store) => {
                let when = faux::When::new("other", mock_store.get_mut().unwrap());
                when.safe_then(f);
            }
        }
    }
}

impl _FauxOriginal_Foo {
    pub fn bar(&self, _: i32) -> impl Fn(i32) -> f64 {
        dbg!("WHAT");
        |_| {
            println!("QUACK");
            3_f64
        }
    }

    pub fn other(&self, _: f64) -> impl MyTrait {
        MyStruct {}
    }
}

use faux::when;

#[test]
fn test() {
    let mut foo = Foo::faux();
    let x = 3 + 3;
    foo._when_bar(move |i| move |j| (x + i + j + 3) as f64);
    let f = foo.bar(5);
    assert_eq!(f(2), 11.0);

    // when! {
    //     foo.bar.safe_then(|_| {})
    // };
    // when!(foo.bar, |_| {});
    // when!(foo.bar, |_| {});
}

// #[test]
// fn generic() {
//     let mut foo = Foo::faux();
//     when!(foo.foo).safe_then(|add_one| add_one(2) + 5);
//     assert_eq!(foo.foo(|i| i + 1), 8);
// }
