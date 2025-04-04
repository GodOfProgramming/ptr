use std::{
  cell::RefCell,
  fmt::{Debug, Display, Error, Formatter},
  ops::{Deref, DerefMut},
  ptr,
  rc::Rc,
};

pub mod prelude {
  pub use super::*;
}

pub struct ConstPtr<T: ?Sized>(*const T);

impl<T> Default for ConstPtr<T> {
  fn default() -> Self {
    Self(ptr::null())
  }
}

impl<T> ConstPtr<T> {
  pub fn new(t: &T) -> Self {
    Self(t)
  }

  pub fn raw(&self) -> *const T {
    self.0
  }

  pub fn null(&self) -> bool {
    self.0.is_null()
  }

  pub fn present(&self) -> bool {
    !self.null()
  }

  pub fn clear(&mut self) {
    self.0 = ptr::null();
  }
}

impl<T> AsRef<T> for ConstPtr<T> {
  fn as_ref(&self) -> &T {
    unsafe { &*self.raw() }
  }
}

impl<T> Clone for ConstPtr<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> Copy for ConstPtr<T> {}

impl<T> Deref for ConstPtr<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0 }
  }
}

impl<T> From<MutPtr<T>> for ConstPtr<T> {
  fn from(ptr: MutPtr<T>) -> Self {
    Self(ptr.raw())
  }
}

impl<T> From<Rc<T>> for ConstPtr<T> {
  fn from(ptr: Rc<T>) -> Self {
    Self(ptr.as_ref())
  }
}

impl<T> From<&Box<T>> for ConstPtr<T> {
  fn from(ptr: &Box<T>) -> Self {
    Self(ptr.as_ref())
  }
}

impl<T> From<*const T> for ConstPtr<T> {
  fn from(value: *const T) -> Self {
    Self(value)
  }
}

pub struct MutPtr<T: ?Sized>(*mut T);

impl<T> Default for MutPtr<T> {
  fn default() -> Self {
    Self(ptr::null_mut())
  }
}

impl<T> Clone for MutPtr<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> Copy for MutPtr<T> {}

impl<T> MutPtr<T> {
  pub fn new(t: &mut T) -> Self {
    Self(t)
  }

  pub fn raw(&self) -> *mut T {
    self.0
  }

  pub fn null(&self) -> bool {
    self.0.is_null()
  }

  pub fn present(&self) -> bool {
    !self.null()
  }

  pub fn clear(&mut self) {
    self.0 = ptr::null_mut();
  }
}

impl<T> AsRef<T> for MutPtr<T> {
  fn as_ref(&self) -> &T {
    unsafe { &*self.raw() }
  }
}

impl<T> AsMut<T> for MutPtr<T> {
  fn as_mut(&mut self) -> &mut T {
    unsafe { &mut *self.raw() }
  }
}

impl<T> Deref for MutPtr<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    unsafe { &*self.0 }
  }
}

impl<T> DerefMut for MutPtr<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.0 }
  }
}

impl<T> From<&mut Box<T>> for MutPtr<T> {
  fn from(ptr: &mut Box<T>) -> Self {
    Self(ptr.as_mut())
  }
}

impl<T> From<Rc<RefCell<T>>> for MutPtr<T> {
  fn from(ptr: Rc<RefCell<T>>) -> Self {
    Self(ptr.as_ptr())
  }
}

impl<T> From<*mut T> for MutPtr<T> {
  fn from(value: *mut T) -> Self {
    Self(value)
  }
}

pub trait AsPtr {
  fn as_ptr(&self) -> ConstPtr<Self>
  where
    Self: Sized,
  {
    ConstPtr(self)
  }

  fn as_ptr_mut(&mut self) -> MutPtr<Self>
  where
    Self: Sized,
  {
    MutPtr(self)
  }
}

pub struct SmartPtr<T> {
  ptr: MutPtr<T>,
  rc: MutPtr<usize>,
}

impl<T> SmartPtr<T> {
  pub fn new(item: T) -> Self {
    let ptr = MutPtr::new(Box::leak(Box::new(item)));
    let rc = Self::new_ref_count();

    Self { ptr, rc }
  }

  pub fn valid(&self) -> bool {
    self.ptr.present() && self.rc.present() && *self.rc > 0
  }

  pub fn access(&self) -> &T {
    &self.ptr
  }

  pub fn access_mut(&mut self) -> &mut T {
    &mut self.ptr
  }

  #[cfg(test)]
  pub fn count(&self) -> usize {
    *self.rc
  }

  fn new_ref_count() -> MutPtr<usize> {
    MutPtr::new(Box::leak(Box::new(1usize)))
  }
}

impl<T> Default for SmartPtr<T>
where
  T: Default,
{
  fn default() -> Self {
    Self::new(T::default())
  }
}

impl<T> Drop for SmartPtr<T> {
  fn drop(&mut self) {
    if self.valid() {
      *self.rc -= 1;
      if *self.rc == 0 {
        unsafe {
          let _ = Box::from_raw(self.ptr.raw());
          let _ = Box::from_raw(self.rc.raw());
        }
      }
    }
  }
}

impl<T> Deref for SmartPtr<T> {
  type Target = MutPtr<T>;
  fn deref(&self) -> &Self::Target {
    &self.ptr
  }
}

impl<T> DerefMut for SmartPtr<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.ptr
  }
}

impl<T> Clone for SmartPtr<T> {
  fn clone(&self) -> Self {
    let ptr = self.ptr;
    let mut rc = self.rc;

    *rc += 1;

    Self { ptr, rc }
  }
}

impl<T: Debug> Debug for SmartPtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
    self.ptr.fmt(f)
  }
}

impl<T: Display> Display for SmartPtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
    self.ptr.fmt(f)
  }
}

impl<T: PartialEq> PartialEq<SmartPtr<T>> for SmartPtr<T> {
  fn eq(&self, other: &Self) -> bool {
    self.ptr.eq(other)
  }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for SmartPtr<T>
where
  T: serde::Serialize,
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    <T as serde::Serialize>::serialize(self, serializer)
  }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for SmartPtr<T>
where
  T: serde::Deserialize<'de>,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let value = <T as serde::Deserialize>::deserialize(deserializer)?;
    Ok(SmartPtr::new(value))
  }
}

#[cfg(test)]
mod tests;
