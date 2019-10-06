use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[doc(hidden)]
#[derive(Default)]
pub struct Quack {
    one_time_mocks: HashMap<TypeId, Box<dyn FnOnce(()) -> ()>>,
    safe_one_time_mocks: HashMap<TypeId, Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>>,
}

impl Quack {
    pub fn mock_once_safe<I: 'static, O: 'static>(
        &mut self,
        id: TypeId,
        mock: impl FnOnce(I) -> O + 'static,
    ) {
        let mock = |input: Box<dyn Any>| {
            let input = *(input.downcast().unwrap());
            let output = mock(input);
            Box::new(output) as Box<dyn Any>
        };
        self.safe_one_time_mocks.insert(id, Box::new(mock));
    }

    pub unsafe fn mock_once<I, O>(&mut self, id: TypeId, mock: impl FnOnce(I) -> O) {
        let mock = Box::new(mock) as Box<dyn FnOnce(_) -> _>;
        let mock = std::mem::transmute(mock);
        self.one_time_mocks.insert(id, mock);
    }

    pub unsafe fn call_mock<I, O>(&mut self, id: &TypeId, input: I) -> Option<O> {
        let mock = self.one_time_mocks.remove(&id)?;
        let mock: Box<dyn FnOnce(I) -> O> = std::mem::transmute(mock);
        Some(mock(input))
    }

    pub fn safe_call_mock<I: 'static, O: 'static>(&mut self, id: &TypeId, input: I) -> Option<O> {
        let mock = self.safe_one_time_mocks.remove(&id)?;
        let output = mock(Box::new(input) as Box<dyn Any>);
        Some(*(output.downcast().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_once() {
        let mut quack = Quack::default();
        let mocked_fn = || {};
        unsafe { quack.mock_once(mocked_fn.type_id(), |i: i32| i + 10) };
        let result: Option<i32> = unsafe { quack.call_mock(&mocked_fn.type_id(), 30_i32) };
        assert_eq!(result, Some(40));
    }

    #[test]
    fn no_mock() {
        let mut quack = Quack::default();
        let mocked_fn = || {};
        let not_mocked_fn = || {};
        unsafe {
            quack.mock_once(mocked_fn.type_id(), |i: i32| i + 10);
        }
        let result: Option<i32> = unsafe { quack.call_mock(&not_mocked_fn.type_id(), 30_i32) };
        assert_eq!(result, None);
    }

    #[test]
    fn mock_ref_input() {
        let mut quack = Quack::default();
        let mocked_fn = || {};
        unsafe {
            quack.mock_once(mocked_fn.type_id(), |_: &i32| 3.3_f64);
        }
        let x = 3_i32 + 3_i32;
        let result: Option<f64> = unsafe { quack.call_mock(&mocked_fn.type_id(), &x) };
        assert_eq!(result, Some(3.3_f64));
    }
}
