use parking_lot::Mutex;

use super::Stub;

pub struct Mock<'a, I, O, const N: usize> {
    pub(super) stubs: Vec<Mutex<Stub<'a, I, O, N>>>,
}

impl<'a, I, O, const N: usize> Mock<'a, I, O, N> {
    pub fn new() -> Self {
        Self { stubs: vec![] }
    }

    pub fn call(&self, mut input: I) -> Result<O, Vec<String>> {
        let mut errors = vec![];

        for stub in self.stubs.iter().rev() {
            match stub.lock().call(input) {
                Err((i, e)) => {
                    errors.push(format!("✗ {}", e));
                    input = i
                }
                Ok(o) => return Ok(o),
            }
        }

        Err(errors)
    }

    pub fn add_stub(&mut self, stub: Stub<'a, I, O, N>) {
        self.stubs.push(Mutex::new(stub))
    }
}
