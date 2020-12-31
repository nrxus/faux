use faux;

trait UsesPin {
    fn do_stuff(self: std::pin::Pin<&mut Self>);
}

#[faux::create]
pub struct Foo {
    a: u32,
}

#[faux::methods(self_type="Pin")]
impl UsesPin for Foo {
    fn do_stuff(self: std::pin::Pin<&mut Self>) {
    }
}

fn main() {}
