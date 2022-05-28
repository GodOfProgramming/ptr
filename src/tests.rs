
use super::*;

#[test]
fn smart_pointer_drops() {
  struct Test {
    f: Box<dyn FnMut()>,
  }

  impl Test {
    fn new(f: Box<dyn FnMut()>) -> Self {
      Self { f }
    }
  }

  impl Drop for Test {
    fn drop(&mut self) {
      (*self.f)();
    }
  }

  let mut dropped = false;

  {
    let mut dropped_ptr = MutPtr::new(&mut dropped);
    let test = Test::new(Box::new(move || {
      *dropped_ptr = true;
    }));
    assert!(!dropped);

    let test_ptr = SmartPtr::new(test);
    assert!(!dropped);

    assert_eq!(test_ptr.count(), 1);

    {
      let test_ptr_cpy = test_ptr.clone();
      assert!(!dropped);

      assert_eq!(test_ptr.count(), 2);
      assert_eq!(test_ptr_cpy.count(), 2);
    }

    assert_eq!(test_ptr.count(), 1);
    assert!(!dropped);
  }

  assert!(dropped);
}
