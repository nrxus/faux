use faux;

#[faux::create]
pub struct Foo {}

#[faux::methods]
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

    async fn private(&self) -> i32 {
        3
    }
}

#[test]
fn real_instance() {
    let fetched = futures::executor::block_on(async {
	let foo = Foo::new().await;
	foo.fetch().await
    });

    assert_eq!(fetched, 3);
}

#[test]
fn mocked() {
    let mut foo = Foo::faux();
    faux::when!(foo.fetch).safe_then(|_| { 10 });
    let fetched = futures::executor::block_on(foo.fetch());
    assert_eq!(fetched, 10);
}
